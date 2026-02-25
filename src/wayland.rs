use anyhow::{Context, Result};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{SeatHandler, SeatState},
    shell::wlr_layer::{
        Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
        LayerSurfaceConfigure,
    },
    shell::WaylandSurface,
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use smithay_client_toolkit::seat::pointer::{PointerEvent, PointerEventKind, PointerHandler};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
    Connection, QueueHandle,
};

use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::canvas::{Canvas, FontState};
use crate::config::{self, ClockConfig, FaceMode};
use crate::ipc;
use crate::renderer::{self, ClockState};
use crate::time_utils;

pub struct Clockie {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    compositor_state: CompositorState,
    layer_shell: LayerShell,
    shm: Shm,
    pool: SlotPool,

    layer_surface: LayerSurface,
    current_output: Option<wl_output::WlOutput>,
    width: u32,
    height: u32,
    configured: bool,
    needs_redraw: bool,

    config: ClockConfig,
    config_path: PathBuf,
    compact: bool,
    font: FontState,

    // Pointer / drag-to-move
    pointer: Option<wl_pointer::WlPointer>,
    locked: bool,
    dragging: bool,
    drag_start: (f64, f64),
    drag_margins: (i32, i32, i32, i32), // (top, right, bottom, left) at drag start
    anchor: Anchor,

    // IPC
    ipc_listener: UnixListener,
    ipc_socket_path: PathBuf,

    // Pending initial output move (applied after first configure when outputs are known)
    pending_output_move: Option<String>,

    should_quit: bool,
}

pub fn run(config: ClockConfig, config_path: PathBuf, socket_override: Option<PathBuf>) -> Result<()> {
    let conn = Connection::connect_to_env().context(
        "Failed to connect to Wayland. Ensure a Wayland compositor with wlr-layer-shell support is running."
    )?;

    let (globals, mut event_queue) = registry_queue_init(&conn)
        .context("Failed to initialize Wayland registry")?;
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh)
        .context("wl_compositor not available")?;
    let layer_shell = LayerShell::bind(&globals, &qh)
        .context("wlr-layer-shell not available. Your compositor must support the wlr_layer_shell_v1 protocol.")?;
    let shm = Shm::bind(&globals, &qh)
        .context("wl_shm not available")?;

    let surface = compositor.create_surface(&qh);

    // Parse layer
    let layer = match config.window.layer.as_str() {
        "background" => Layer::Background,
        "bottom" => Layer::Bottom,
        "top" => Layer::Top,
        "overlay" => Layer::Overlay,
        _ => Layer::Top,
    };

    let layer_surface = layer_shell.create_layer_surface(&qh, surface, layer, Some("clockie"), None);

    // Compute initial size from content
    let compact = config.window.compact;
    let font = FontState::new(&config.clock.font);
    let (init_w, init_h) = renderer::compute_size(&config, &font, compact);

    // Set size
    layer_surface.set_size(init_w, init_h);

    // Parse and set anchor
    let mut anchor = Anchor::empty();
    for part in config.window.anchor.split_whitespace() {
        match part.to_lowercase().as_str() {
            "top" => anchor |= Anchor::TOP,
            "bottom" => anchor |= Anchor::BOTTOM,
            "left" => anchor |= Anchor::LEFT,
            "right" => anchor |= Anchor::RIGHT,
            _ => {}
        }
    }
    layer_surface.set_anchor(anchor);

    // Set margins
    layer_surface.set_margin(
        config.window.margin_top,
        config.window.margin_right,
        config.window.margin_bottom,
        config.window.margin_left,
    );

    // No exclusive zone
    layer_surface.set_exclusive_zone(0);

    // No keyboard grab
    layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);

    // Commit initial state
    layer_surface.commit();

    let pool = SlotPool::new(
        (init_w * init_h * 4) as usize,
        &shm,
    ).context("Failed to create SHM pool")?;

    // IPC setup
    let ipc_socket_path = ipc::socket_path(socket_override.as_ref());
    let ipc_listener = ipc::create_listener(&ipc_socket_path)?;

    let pending_output_move = config.window.output.clone();

    let mut clockie = Clockie {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        compositor_state: compositor,
        layer_shell,
        shm,
        pool,
        layer_surface,
        current_output: None,
        width: init_w,
        height: init_h,
        configured: false,
        needs_redraw: true,
        config,
        config_path,
        compact,
        font,
        pointer: None,
        locked: false,
        dragging: false,
        drag_start: (0.0, 0.0),
        drag_margins: (0, 0, 0, 0),
        anchor,
        ipc_listener,
        ipc_socket_path,
        pending_output_move,
        should_quit: false,
    };

    // Roundtrip to populate output state before applying pending output move
    event_queue.roundtrip(&mut clockie)?;

    // If we have a configured output, move to it now that outputs are known
    if clockie.pending_output_move.is_some() {
        let qh = event_queue.handle();
        clockie.apply_pending_output_move(&qh);
    }

    // Signal handling
    let running = Arc::new(AtomicBool::new(true));
    {
        let r = running.clone();
        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
        }).expect("Failed to set signal handler");
    }

    // Main event loop
    let mut last_second = 0u32;

    loop {
        if clockie.should_quit || !running.load(Ordering::SeqCst) {
            break;
        }

        // Dispatch Wayland events (blocking with timeout)
        event_queue.flush()?;
        if let Some(guard) = event_queue.prepare_read() {
            // Use a short timeout so we can check the timer
            let fd = guard.connection_fd();
            let mut fds = [nix::poll::PollFd::new(fd, nix::poll::PollFlags::POLLIN)];
            let _ = nix::poll::poll(&mut fds, nix::poll::PollTimeout::from(100u16));
            if fds[0].revents().map_or(false, |r| r.contains(nix::poll::PollFlags::POLLIN)) {
                guard.read()?;
            } else {
                drop(guard);
            }
        }
        event_queue.dispatch_pending(&mut clockie)?;

        // Check for IPC connections
        clockie.poll_ipc(&qh);

        // 1Hz timer: check if second changed
        let now = chrono::Local::now();
        let current_second = chrono::Timelike::second(&now);
        if current_second != last_second {
            last_second = current_second;
            clockie.needs_redraw = true;
        }

        // Redraw if needed
        if clockie.configured && clockie.needs_redraw {
            clockie.draw(&qh);
            clockie.needs_redraw = false;
        }
    }

    // Cleanup
    ipc::cleanup_socket(&clockie.ipc_socket_path);

    Ok(())
}

/// Format an Anchor bitfield back to a string like "top right".
fn format_anchor(anchor: Anchor) -> String {
    let mut parts = Vec::new();
    if anchor.contains(Anchor::TOP) { parts.push("top"); }
    if anchor.contains(Anchor::BOTTOM) { parts.push("bottom"); }
    if anchor.contains(Anchor::LEFT) { parts.push("left"); }
    if anchor.contains(Anchor::RIGHT) { parts.push("right"); }
    parts.join(" ")
}

/// Direction for finding adjacent outputs.
#[derive(Debug, Clone, Copy)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Clockie {
    /// Get the name of the current output, if known.
    fn get_output_name(&self) -> Option<String> {
        self.current_output.as_ref().and_then(|wl_out| {
            self.output_state.info(wl_out).and_then(|info| info.name.clone())
        })
    }

    /// Recreate the layer surface on a different output.
    fn recreate_surface(&mut self, qh: &QueueHandle<Self>, target_output: Option<&wl_output::WlOutput>) {
        // Parse layer
        let layer = match self.config.window.layer.as_str() {
            "background" => Layer::Background,
            "bottom" => Layer::Bottom,
            "top" => Layer::Top,
            "overlay" => Layer::Overlay,
            _ => Layer::Top,
        };

        // Create new surface
        let surface = self.compositor_state.create_surface(qh);
        let new_layer_surface = self.layer_shell.create_layer_surface(
            qh, surface, layer, Some("clockie"), target_output,
        );

        // Configure identically
        new_layer_surface.set_size(self.width, self.height);
        new_layer_surface.set_anchor(self.anchor);
        new_layer_surface.set_margin(
            self.config.window.margin_top,
            self.config.window.margin_right,
            self.config.window.margin_bottom,
            self.config.window.margin_left,
        );
        new_layer_surface.set_exclusive_zone(0);
        new_layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
        new_layer_surface.commit();

        // Replace old surface (dropping it destroys the old one)
        self.layer_surface = new_layer_surface;
        self.current_output = target_output.cloned();
        self.configured = false;
        self.needs_redraw = true;

        log::info!("Recreated surface on output: {:?}", self.get_output_name());
    }

    /// Find an adjacent output in the given direction relative to the current output.
    fn find_adjacent_output(&self, direction: Direction) -> Option<wl_output::WlOutput> {
        let current = self.current_output.as_ref()?;
        let current_info = self.output_state.info(current)?;
        let (cx, cy) = current_info.logical_position.unwrap_or((0, 0));
        let (cw, ch) = current_info.logical_size.unwrap_or((0, 0));

        let mut best: Option<(wl_output::WlOutput, i32)> = None;

        for wl_output in self.output_state.outputs() {
            if &wl_output == current {
                continue;
            }
            let Some(info) = self.output_state.info(&wl_output) else { continue };
            let (ox, oy) = info.logical_position.unwrap_or((0, 0));
            let (ow, oh) = info.logical_size.unwrap_or((0, 0));

            let is_adjacent = match direction {
                Direction::Right => {
                    // Output is to the right: its left edge touches our right edge
                    // and there's vertical overlap
                    ox == cx + cw && oy < cy + ch && oy + oh > cy
                }
                Direction::Left => {
                    ox + ow == cx && oy < cy + ch && oy + oh > cy
                }
                Direction::Down => {
                    oy == cy + ch && ox < cx + cw && ox + ow > cx
                }
                Direction::Up => {
                    oy + oh == cy && ox < cx + cw && ox + ow > cx
                }
            };

            if is_adjacent {
                // Use distance from current center as tie-breaker
                let dist = match direction {
                    Direction::Right | Direction::Left => ((oy + oh / 2) - (cy + ch / 2)).abs(),
                    Direction::Up | Direction::Down => ((ox + ow / 2) - (cx + cw / 2)).abs(),
                };
                if best.as_ref().map_or(true, |(_, d)| dist < *d) {
                    best = Some((wl_output, dist));
                }
            }
        }

        best.map(|(o, _)| o)
    }

    /// Find output by name from the output state.
    fn find_output_by_name(&self, name: &str) -> Option<wl_output::WlOutput> {
        for wl_output in self.output_state.outputs() {
            if let Some(info) = self.output_state.info(&wl_output) {
                if info.name.as_deref() == Some(name) {
                    return Some(wl_output);
                }
            }
        }
        None
    }

    /// Find next/prev output by cycling through all outputs.
    fn find_output_cycle(&self, forward: bool) -> Option<wl_output::WlOutput> {
        let outputs: Vec<_> = self.output_state.outputs().collect();
        if outputs.len() <= 1 {
            return None;
        }

        let current_idx = self.current_output.as_ref().and_then(|current| {
            outputs.iter().position(|o| o == current)
        }).unwrap_or(0);

        let next_idx = if forward {
            (current_idx + 1) % outputs.len()
        } else {
            (current_idx + outputs.len() - 1) % outputs.len()
        };

        Some(outputs[next_idx].clone())
    }

    /// Apply a pending output move (used at startup).
    fn apply_pending_output_move(&mut self, qh: &QueueHandle<Self>) {
        if let Some(name) = self.pending_output_move.take() {
            if let Some(target) = self.find_output_by_name(&name) {
                log::info!("Moving to configured output: {}", name);
                self.recreate_surface(qh, Some(&target));
            } else {
                log::warn!("Configured output '{}' not found, staying on default", name);
            }
        }
    }

    /// Recompute window size from content and apply if changed.
    /// Clamps margins so the window stays within the current output.
    fn update_size(&mut self) {
        let (new_w, new_h) = renderer::compute_size(&self.config, &self.font, self.compact);
        if new_w != self.width || new_h != self.height {
            self.width = new_w;
            self.height = new_h;
            self.layer_surface.set_size(self.width, self.height);

            // Clamp margins so the window doesn't overflow the output
            self.clamp_margins();

            self.layer_surface.set_margin(
                self.config.window.margin_top,
                self.config.window.margin_right,
                self.config.window.margin_bottom,
                self.config.window.margin_left,
            );
            self.layer_surface.wl_surface().commit();
        }
        self.needs_redraw = true;
    }

    /// Clamp margins so the window fits within the current output bounds.
    fn clamp_margins(&mut self) {
        let (out_w, out_h) = self.current_output.as_ref()
            .and_then(|o| self.output_state.info(o))
            .and_then(|info| info.logical_size)
            .unwrap_or((0, 0));

        if out_w == 0 || out_h == 0 {
            return;
        }

        let has_left = self.anchor.contains(Anchor::LEFT);
        let has_right = self.anchor.contains(Anchor::RIGHT);
        let has_top = self.anchor.contains(Anchor::TOP);
        let has_bottom = self.anchor.contains(Anchor::BOTTOM);

        // Horizontal: margin_left + width + margin_right <= output_width
        if has_left && !has_right {
            let max = (out_w as u32).saturating_sub(self.width) as i32;
            self.config.window.margin_left = self.config.window.margin_left.clamp(0, max);
        } else if has_right && !has_left {
            let max = (out_w as u32).saturating_sub(self.width) as i32;
            self.config.window.margin_right = self.config.window.margin_right.clamp(0, max);
        }

        // Vertical: margin_top + height + margin_bottom <= output_height
        if has_top && !has_bottom {
            let max = (out_h as u32).saturating_sub(self.height) as i32;
            self.config.window.margin_top = self.config.window.margin_top.clamp(0, max);
        } else if has_bottom && !has_top {
            let max = (out_h as u32).saturating_sub(self.height) as i32;
            self.config.window.margin_bottom = self.config.window.margin_bottom.clamp(0, max);
        }
    }

    fn draw(&mut self, qh: &QueueHandle<Self>) {
        let width = self.width;
        let height = self.height;

        if width == 0 || height == 0 { return; }

        let stride = width as i32 * 4;
        let buf_size = (stride * height as i32) as usize;

        // Ensure pool is big enough
        if self.pool.len() < buf_size {
            self.pool.resize(buf_size).expect("Failed to resize SHM pool");
        }

        let (buffer, canvas_data) = self.pool
            .create_buffer(width as i32, height as i32, stride, wl_shm::Format::Argb8888)
            .expect("Failed to create buffer");

        // Render to canvas
        let mut canvas = Canvas::new(width, height);
        let time = time_utils::current_time(&self.config.clock.date_format);
        let battery = if self.config.battery.enabled {
            crate::battery::read_battery()
        } else {
            None
        };

        let state = ClockState {
            config: self.config.clone(),
            time,
            compact: self.compact,
            battery,
        };

        renderer::render(&mut canvas, &state, &self.font);

        // Apply window opacity
        let opacity = self.config.window.opacity;
        if opacity < 1.0 {
            let data = canvas.pixmap.data_mut();
            let scale = (opacity * 255.0) as u32;
            for i in (0..data.len()).step_by(4) {
                data[i + 3] = ((data[i + 3] as u32 * scale) / 255) as u8;
            }
        }

        // Copy pixels with RGBA→BGRA swizzle
        let pixels = canvas.pixels_argb8888();
        canvas_data[..pixels.len()].copy_from_slice(&pixels);

        // Attach and commit
        let surface = self.layer_surface.wl_surface();
        buffer.attach_to(surface).expect("Failed to attach buffer");
        surface.damage_buffer(0, 0, width as i32, height as i32);
        surface.frame(qh, surface.clone());
        surface.commit();
    }

    fn poll_ipc(&mut self, qh: &QueueHandle<Self>) {
        loop {
            match self.ipc_listener.accept() {
                Ok((stream, _)) => {
                    self.handle_ipc_connection(stream, qh);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    log::warn!("IPC accept error: {}", e);
                    break;
                }
            }
        }
    }

    fn handle_ipc_connection(&mut self, stream: std::os::unix::net::UnixStream, qh: &QueueHandle<Self>) {
        let cmd = match ipc::read_command(&stream) {
            Ok(cmd) => cmd,
            Err(e) => {
                log::warn!("IPC read error: {}", e);
                return;
            }
        };

        let response = self.handle_command(cmd, qh);
        let mut stream = stream;
        if let Err(e) = ipc::write_response(&mut stream, &response) {
            log::warn!("IPC write error: {}", e);
        }
    }

    fn handle_command(&mut self, cmd: ipc::IpcCommand, qh: &QueueHandle<Self>) -> ipc::IpcResponse {
        match cmd {
            ipc::IpcCommand::SetFace { face } => {
                match face.as_str() {
                    "digital" => {
                        self.config.clock.face = FaceMode::Digital;
                        self.update_size();
                        ipc::IpcResponse::ok()
                    }
                    "analogue" => {
                        self.config.clock.face = FaceMode::Analogue;
                        self.update_size();
                        ipc::IpcResponse::ok()
                    }
                    _ => ipc::IpcResponse::err(format!("Unknown face: {}", face)),
                }
            }
            ipc::IpcCommand::ToggleFace => {
                self.config.clock.face = self.config.clock.face.toggle();
                self.update_size();
                ipc::IpcResponse::ok()
            }
            ipc::IpcCommand::SetCompact { compact } => {
                self.compact = compact;
                self.update_size();
                ipc::IpcResponse::ok()
            }
            ipc::IpcCommand::ToggleCompact => {
                self.compact = !self.compact;
                self.update_size();
                ipc::IpcResponse::ok()
            }
            ipc::IpcCommand::SetFontSize { size } => {
                self.config.clock.font_size = size.max(10.0);
                self.update_size();
                ipc::IpcResponse::ok()
            }
            ipc::IpcCommand::SetDiameter { diameter } => {
                self.config.clock.diameter = diameter.max(40);
                self.update_size();
                ipc::IpcResponse::ok()
            }
            ipc::IpcCommand::ScaleBy { delta } => {
                match self.config.clock.face {
                    FaceMode::Digital => {
                        self.config.clock.font_size = (self.config.clock.font_size + delta as f32).max(10.0);
                    }
                    FaceMode::Analogue => {
                        self.config.clock.diameter = (self.config.clock.diameter as i32 + delta).max(40) as u32;
                    }
                }
                self.update_size();
                ipc::IpcResponse::ok()
            }
            ipc::IpcCommand::SetLocked { locked } => {
                self.locked = locked;
                ipc::IpcResponse::ok()
            }
            ipc::IpcCommand::ToggleLocked => {
                self.locked = !self.locked;
                ipc::IpcResponse::ok()
            }
            ipc::IpcCommand::MoveToOutput { name } => {
                let target = match name.as_str() {
                    "next" => self.find_output_cycle(true),
                    "prev" => self.find_output_cycle(false),
                    _ => self.find_output_by_name(&name),
                };
                match target {
                    Some(output) => {
                        self.recreate_surface(qh, Some(&output));
                        // Persist the output name
                        let output_name = self.get_output_name().unwrap_or_else(|| name.clone());
                        self.config.window.output = Some(output_name.clone());
                        config::save_output_to_config(&self.config_path, &output_name);
                        ipc::IpcResponse::ok()
                    }
                    None => ipc::IpcResponse::err(format!("Output '{}' not found", name)),
                }
            }
            ipc::IpcCommand::ReloadConfig => {
                match config::load_config(&self.config_path) {
                    Ok(new_config) => {
                        // Preserve runtime state
                        let face = self.config.clock.face;
                        let compact = self.compact;

                        // Apply anchor
                        let mut anchor = Anchor::empty();
                        for part in new_config.window.anchor.split_whitespace() {
                            match part.to_lowercase().as_str() {
                                "top" => anchor |= Anchor::TOP,
                                "bottom" => anchor |= Anchor::BOTTOM,
                                "left" => anchor |= Anchor::LEFT,
                                "right" => anchor |= Anchor::RIGHT,
                                _ => {}
                            }
                        }
                        self.layer_surface.set_anchor(anchor);
                        self.anchor = anchor;

                        // Apply margins
                        self.layer_surface.set_margin(
                            new_config.window.margin_top,
                            new_config.window.margin_right,
                            new_config.window.margin_bottom,
                            new_config.window.margin_left,
                        );

                        self.config = new_config;
                        self.config.clock.face = face;
                        self.compact = compact;
                        self.font = FontState::new(&self.config.clock.font);

                        // Recompute size from new config
                        self.update_size();
                        // Commit geometry changes
                        self.layer_surface.wl_surface().commit();
                        ipc::IpcResponse::ok()
                    }
                    Err(e) => ipc::IpcResponse::err(format!("Config reload failed: {}", e)),
                }
            }
            ipc::IpcCommand::GetState => {
                let face = match self.config.clock.face {
                    FaceMode::Digital => "digital",
                    FaceMode::Analogue => "analogue",
                };
                let output_name = self.get_output_name();
                ipc::IpcResponse::state(
                    face,
                    self.compact,
                    self.width,
                    self.height,
                    self.config.clock.font_size,
                    self.config.clock.diameter,
                    &self.config_path.to_string_lossy(),
                    self.locked,
                    output_name.as_deref(),
                )
            }
            ipc::IpcCommand::Quit => {
                self.should_quit = true;
                ipc::IpcResponse::ok()
            }
        }
    }
}

// SCTK handler implementations

impl CompositorHandler for Clockie {
    fn scale_factor_changed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, _new_factor: i32) {
        self.needs_redraw = true;
    }

    fn transform_changed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, _new_transform: wl_output::Transform) {
        self.needs_redraw = true;
    }

    fn frame(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, _time: u32) {
        // Frame callback received
    }

    fn surface_enter(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, output: &wl_output::WlOutput) {
        self.current_output = Some(output.clone());
        if let Some(info) = self.output_state.info(output) {
            log::info!("Surface entered output: {:?}", info.name);
        }
    }
    fn surface_leave(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, output: &wl_output::WlOutput) {
        if self.current_output.as_ref() == Some(output) {
            self.current_output = None;
        }
    }
}

impl LayerShellHandler for Clockie {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.should_quit = true;
    }

    fn configure(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface, configure: LayerSurfaceConfigure, _serial: u32) {
        if configure.new_size.0 > 0 {
            self.width = configure.new_size.0;
        }
        if configure.new_size.1 > 0 {
            self.height = configure.new_size.1;
        }
        self.configured = true;
        self.needs_redraw = true;
    }
}

impl OutputHandler for Clockie {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}
    fn update_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}
}

impl SeatHandler for Clockie {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {}
    fn new_capability(&mut self, _conn: &Connection, qh: &QueueHandle<Self>, seat: wl_seat::WlSeat, capability: SeatCapability) {
        if capability == SeatCapability::Pointer && self.pointer.is_none() {
            self.pointer = Some(self.seat_state.get_pointer(qh, &seat).expect("Failed to get pointer"));
        }
    }
    fn remove_capability(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat, capability: SeatCapability) {
        if capability == SeatCapability::Pointer {
            if let Some(pointer) = self.pointer.take() {
                pointer.release();
            }
        }
    }
    fn remove_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {}
}

use smithay_client_toolkit::seat::Capability as SeatCapability;

impl ShmHandler for Clockie {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ProvidesRegistryState for Clockie {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers![OutputState, SeatState];
}

const BTN_LEFT: u32 = 0x110;

impl PointerHandler for Clockie {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        for event in events {
            match event.kind {
                PointerEventKind::Press { button, .. } if button == BTN_LEFT => {
                    if !self.locked {
                        self.dragging = true;
                        self.drag_start = event.position;
                        self.drag_margins = (
                            self.config.window.margin_top,
                            self.config.window.margin_right,
                            self.config.window.margin_bottom,
                            self.config.window.margin_left,
                        );
                    }
                }
                PointerEventKind::Motion { .. } if self.dragging => {
                    let dx = event.position.0 - self.drag_start.0;
                    let dy = event.position.1 - self.drag_start.1;

                    let has_left = self.anchor.contains(Anchor::LEFT);
                    let has_right = self.anchor.contains(Anchor::RIGHT);
                    let has_top = self.anchor.contains(Anchor::TOP);
                    let has_bottom = self.anchor.contains(Anchor::BOTTOM);

                    // Horizontal
                    if has_left && !has_right {
                        self.config.window.margin_left = (self.drag_margins.3 + dx as i32).max(0);
                    } else if has_right && !has_left {
                        self.config.window.margin_right = (self.drag_margins.1 - dx as i32).max(0);
                    }

                    // Vertical
                    if has_top && !has_bottom {
                        self.config.window.margin_top = (self.drag_margins.0 + dy as i32).max(0);
                    } else if has_bottom && !has_top {
                        self.config.window.margin_bottom = (self.drag_margins.2 - dy as i32).max(0);
                    }

                    self.layer_surface.set_margin(
                        self.config.window.margin_top,
                        self.config.window.margin_right,
                        self.config.window.margin_bottom,
                        self.config.window.margin_left,
                    );
                    self.layer_surface.wl_surface().commit();
                }
                PointerEventKind::Release { button, .. } if button == BTN_LEFT => {
                    if self.dragging {
                        self.dragging = false;
                        let current = (
                            self.config.window.margin_top,
                            self.config.window.margin_right,
                            self.config.window.margin_bottom,
                            self.config.window.margin_left,
                        );
                        if current != self.drag_margins {
                            config::save_margins_to_config(
                                &self.config_path,
                                current.0,
                                current.1,
                                current.2,
                                current.3,
                            );
                        }
                    }
                }
                PointerEventKind::Leave { .. } => {
                    if self.dragging {
                        self.dragging = false;

                        let has_left = self.anchor.contains(Anchor::LEFT);
                        let has_right = self.anchor.contains(Anchor::RIGHT);
                        let has_top = self.anchor.contains(Anchor::TOP);
                        let has_bottom = self.anchor.contains(Anchor::BOTTOM);

                        // Detect which direction the clock was dragged to the edge
                        // A margin is "at edge" if it's 0 and wasn't 0 at drag start
                        let direction = if has_left && !has_right && self.config.window.margin_left == 0 && self.drag_margins.3 > 0 {
                            Some(Direction::Left)
                        } else if has_right && !has_left && self.config.window.margin_right == 0 && self.drag_margins.1 > 0 {
                            Some(Direction::Right)
                        } else if has_top && !has_bottom && self.config.window.margin_top == 0 && self.drag_margins.0 > 0 {
                            Some(Direction::Up)
                        } else if has_bottom && !has_top && self.config.window.margin_bottom == 0 && self.drag_margins.2 > 0 {
                            Some(Direction::Down)
                        } else {
                            // Also check: margin was already 0 at drag start but we're leaving in that direction
                            // This handles the case where the clock was already at the edge
                            if has_left && !has_right && self.config.window.margin_left == 0 {
                                Some(Direction::Left)
                            } else if has_right && !has_left && self.config.window.margin_right == 0 {
                                Some(Direction::Right)
                            } else if has_top && !has_bottom && self.config.window.margin_top == 0 {
                                Some(Direction::Up)
                            } else if has_bottom && !has_top && self.config.window.margin_bottom == 0 {
                                Some(Direction::Down)
                            } else {
                                None
                            }
                        };

                        let moved = if let Some(dir) = direction {
                            if let Some(target) = self.find_adjacent_output(dir) {
                                // Set margin on the arriving edge to 0, keep perpendicular margins
                                match dir {
                                    Direction::Left => {
                                        // Arriving from the right side of the new output
                                        // Flip anchor to right side
                                        self.anchor = (self.anchor & !(Anchor::LEFT)) | Anchor::RIGHT;
                                        self.config.window.anchor = format_anchor(self.anchor);
                                        self.config.window.margin_right = 0;
                                        self.config.window.margin_left = 0;
                                    }
                                    Direction::Right => {
                                        self.anchor = (self.anchor & !(Anchor::RIGHT)) | Anchor::LEFT;
                                        self.config.window.anchor = format_anchor(self.anchor);
                                        self.config.window.margin_left = 0;
                                        self.config.window.margin_right = 0;
                                    }
                                    Direction::Up => {
                                        self.anchor = (self.anchor & !(Anchor::TOP)) | Anchor::BOTTOM;
                                        self.config.window.anchor = format_anchor(self.anchor);
                                        self.config.window.margin_bottom = 0;
                                        self.config.window.margin_top = 0;
                                    }
                                    Direction::Down => {
                                        self.anchor = (self.anchor & !(Anchor::BOTTOM)) | Anchor::TOP;
                                        self.config.window.anchor = format_anchor(self.anchor);
                                        self.config.window.margin_top = 0;
                                        self.config.window.margin_bottom = 0;
                                    }
                                }
                                self.recreate_surface(qh, Some(&target));
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        // Save state
                        let current = (
                            self.config.window.margin_top,
                            self.config.window.margin_right,
                            self.config.window.margin_bottom,
                            self.config.window.margin_left,
                        );
                        if moved || current != self.drag_margins {
                            config::save_margins_to_config(
                                &self.config_path,
                                current.0,
                                current.1,
                                current.2,
                                current.3,
                            );
                        }
                        if moved {
                            if let Some(output_name) = self.get_output_name() {
                                self.config.window.output = Some(output_name.clone());
                                config::save_output_to_config(&self.config_path, &output_name);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

delegate_compositor!(Clockie);
delegate_layer!(Clockie);
delegate_output!(Clockie);
delegate_pointer!(Clockie);
delegate_registry!(Clockie);
delegate_seat!(Clockie);
delegate_shm!(Clockie);
