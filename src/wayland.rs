use anyhow::{Context, Result};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_seat,
    delegate_shm,
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
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_seat, wl_shm, wl_surface},
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
    shm: Shm,
    pool: SlotPool,

    layer_surface: LayerSurface,
    width: u32,
    height: u32,
    configured: bool,
    needs_redraw: bool,

    config: ClockConfig,
    config_path: PathBuf,
    compact: bool,
    font: FontState,

    // IPC
    ipc_listener: UnixListener,
    ipc_socket_path: PathBuf,

    // Pointer state for resize (used when resizable=true)
    #[allow(dead_code)]
    pointer_x: f64,
    #[allow(dead_code)]
    pointer_y: f64,
    #[allow(dead_code)]
    resizing: bool,
    #[allow(dead_code)]
    resize_edge: ResizeEdge,

    should_quit: bool,
}

#[derive(Debug, Default, Clone, Copy)]
#[allow(dead_code)]
struct ResizeEdge {
    left: bool,
    right: bool,
    top: bool,
    bottom: bool,
}

#[allow(dead_code)]
impl ResizeEdge {
    fn any(&self) -> bool {
        self.left || self.right || self.top || self.bottom
    }
}

const MIN_WIDTH: u32 = 120;
const MIN_HEIGHT: u32 = 80;
#[allow(dead_code)]
const EDGE_THRESHOLD: f64 = 10.0;

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

    // Set size
    layer_surface.set_size(config.window.width, config.window.height);

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
        (config.window.width * config.window.height * 4) as usize,
        &shm,
    ).context("Failed to create SHM pool")?;

    // IPC setup
    let ipc_socket_path = ipc::socket_path(socket_override.as_ref());
    let ipc_listener = ipc::create_listener(&ipc_socket_path)?;

    let compact = config.window.compact;
    let font = FontState::new(&config.clock.font);

    let mut clockie = Clockie {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,
        pool,
        layer_surface,
        width: config.window.width,
        height: config.window.height,
        configured: false,
        needs_redraw: true,
        config,
        config_path,
        compact,
        font,
        ipc_listener,
        ipc_socket_path,
        pointer_x: 0.0,
        pointer_y: 0.0,
        resizing: false,
        resize_edge: ResizeEdge::default(),
        should_quit: false,
    };

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
        clockie.poll_ipc();

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

    // Persist size to config
    save_size_to_config(&clockie.config_path, clockie.width, clockie.height);

    Ok(())
}

fn save_size_to_config(path: &PathBuf, width: u32, height: u32) {
    if !path.exists() { return; }
    if let Ok(content) = std::fs::read_to_string(path) {
        // Simple approach: parse, modify, write
        if let Ok(mut value) = content.parse::<toml::Value>() {
            if let Some(window) = value.get_mut("window").and_then(|v| v.as_table_mut()) {
                window.insert("width".into(), toml::Value::Integer(width as i64));
                window.insert("height".into(), toml::Value::Integer(height as i64));
                if let Ok(new_content) = toml::to_string_pretty(&value) {
                    let _ = std::fs::write(path, new_content);
                }
            }
        }
    }
}

impl Clockie {
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
        let state = ClockState {
            config: self.config.clone(),
            time,
            compact: self.compact,
            width,
            height,
        };

        renderer::render(&mut canvas, &state, &self.font);

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

    fn poll_ipc(&mut self) {
        loop {
            match self.ipc_listener.accept() {
                Ok((stream, _)) => {
                    self.handle_ipc_connection(stream);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    log::warn!("IPC accept error: {}", e);
                    break;
                }
            }
        }
    }

    fn handle_ipc_connection(&mut self, stream: std::os::unix::net::UnixStream) {
        let cmd = match ipc::read_command(&stream) {
            Ok(cmd) => cmd,
            Err(e) => {
                log::warn!("IPC read error: {}", e);
                return;
            }
        };

        let response = self.handle_command(cmd);
        let mut stream = stream;
        if let Err(e) = ipc::write_response(&mut stream, &response) {
            log::warn!("IPC write error: {}", e);
        }
    }

    fn handle_command(&mut self, cmd: ipc::IpcCommand) -> ipc::IpcResponse {
        match cmd {
            ipc::IpcCommand::SetFace { face } => {
                match face.as_str() {
                    "digital" => { self.config.clock.face = FaceMode::Digital; self.needs_redraw = true; ipc::IpcResponse::ok() }
                    "analogue" => { self.config.clock.face = FaceMode::Analogue; self.needs_redraw = true; ipc::IpcResponse::ok() }
                    _ => ipc::IpcResponse::err(format!("Unknown face: {}", face)),
                }
            }
            ipc::IpcCommand::ToggleFace => {
                self.config.clock.face = self.config.clock.face.toggle();
                self.needs_redraw = true;
                ipc::IpcResponse::ok()
            }
            ipc::IpcCommand::SetCompact { compact } => {
                self.compact = compact;
                self.needs_redraw = true;
                ipc::IpcResponse::ok()
            }
            ipc::IpcCommand::ToggleCompact => {
                self.compact = !self.compact;
                self.needs_redraw = true;
                ipc::IpcResponse::ok()
            }
            ipc::IpcCommand::SetSize { width, height } => {
                self.width = width.max(MIN_WIDTH);
                self.height = height.max(MIN_HEIGHT);
                self.layer_surface.set_size(self.width, self.height);
                self.layer_surface.commit();
                self.needs_redraw = true;
                ipc::IpcResponse::ok()
            }
            ipc::IpcCommand::ResizeBy { delta } => {
                let aspect = self.width as f64 / self.height as f64;
                let new_w = (self.width as i32 + delta).max(MIN_WIDTH as i32) as u32;
                let new_h = (new_w as f64 / aspect) as u32;
                let new_h = new_h.max(MIN_HEIGHT);
                self.width = new_w;
                self.height = new_h;
                self.layer_surface.set_size(self.width, self.height);
                self.layer_surface.commit();
                self.needs_redraw = true;
                ipc::IpcResponse::ok()
            }
            ipc::IpcCommand::ReloadConfig => {
                match config::load_config(&self.config_path) {
                    Ok(new_config) => {
                        // Preserve runtime state
                        let face = self.config.clock.face;
                        let compact = self.compact;
                        self.config = new_config;
                        self.config.clock.face = face;
                        self.compact = compact;
                        self.font = FontState::new(&self.config.clock.font);
                        self.needs_redraw = true;
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
                ipc::IpcResponse::state(
                    face,
                    self.compact,
                    self.width,
                    self.height,
                    &self.config_path.to_string_lossy(),
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

    fn surface_enter(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, _output: &wl_output::WlOutput) {}
    fn surface_leave(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, _output: &wl_output::WlOutput) {}
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
    fn new_capability(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat, _capability: SeatCapability) {}
    fn remove_capability(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat, _capability: SeatCapability) {}
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

delegate_compositor!(Clockie);
delegate_layer!(Clockie);
delegate_output!(Clockie);
delegate_registry!(Clockie);
delegate_seat!(Clockie);
delegate_shm!(Clockie);

