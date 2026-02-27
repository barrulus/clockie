pub mod analogue;
pub mod battery;
pub mod digital;
pub mod subclock;

use crate::battery::BatteryInfo;
use crate::canvas::{Canvas, FontState};
use crate::config::{ClockConfig, FaceMode};
use crate::time_utils::ClockTime;

/// Resolved contrast information for text rendering.
pub struct ContrastInfo {
    /// The color to use for text (may differ from theme.fg_color when auto-contrast is active).
    pub text_color: [u8; 4],
    /// Whether to draw a contrasting outline around text.
    pub use_outline: bool,
}

pub struct ClockState {
    pub config: ClockConfig,
    pub time: ClockTime,
    pub compact: bool,
    pub battery: Option<BatteryInfo>,
    pub contrast: ContrastInfo,
}

/// Draw text, optionally with a contrasting outline based on ContrastInfo.
pub fn draw_contrast_text(font: &FontState, canvas: &mut Canvas, text: &str, x: f32, y: f32, size: f32, color: [u8; 4], contrast: &ContrastInfo) {
    if contrast.use_outline {
        let outline = outline_color_for(color);
        font.draw_text_outlined(canvas, text, x, y, size, color, outline);
    } else {
        font.draw_text(canvas, text, x, y, size, color);
    }
}

/// Pick a contrasting outline color: dark outline for light text, light for dark.
fn outline_color_for(color: [u8; 4]) -> [u8; 4] {
    let lum = 0.2126 * color[0] as f32 + 0.7152 * color[1] as f32 + 0.0722 * color[2] as f32;
    if lum > 128.0 {
        [0x00, 0x00, 0x00, color[3]]
    } else {
        [0xFF, 0xFF, 0xFF, color[3]]
    }
}

/// Shared sizing constants for subclock text, eliminating duplication across renderers.
#[allow(dead_code)]
pub struct SubclockSizing {
    pub label_size: f32,
    pub time_size: f32,
    pub row_h: f32,
    pub sep_gap: f32,
    pub area_h: f32,
}

impl SubclockSizing {
    /// Compute subclock sizing from a base size (font_size for digital, diameter*0.25 for analogue).
    pub fn from_base(base: f32) -> Self {
        let pad_y = base * 0.25;
        let label_size = (base * 0.33).max(11.0);
        let time_size = (label_size * 1.5).max(16.0);
        let row_h = label_size + time_size + label_size * 0.1;
        let sep_gap = pad_y * 0.5;
        let area_h = sep_gap + row_h + sep_gap;
        Self { label_size, time_size, row_h, sep_gap, area_h }
    }
}

/// Compute the required window dimensions based on config, font, and compact state.
pub fn compute_size(config: &ClockConfig, font: &FontState, compact: bool) -> (u32, u32) {
    match config.clock.face {
        FaceMode::Digital => compute_digital_size(config, font, compact),
        FaceMode::Analogue => compute_analogue_size(config, font, compact),
    }
}

fn compute_digital_size(config: &ClockConfig, font: &FontState, compact: bool) -> (u32, u32) {
    let font_size = config.clock.font_size;
    let time_size = if compact { font_size * 0.7 } else { font_size };
    let pad_x = time_size * 0.4;
    let pad_y = time_size * 0.25;

    // Measure widest possible time string to avoid width jitter
    let widest_time = widest_time_string(config);
    let (time_w, _) = font.measure_text(&widest_time, time_size);

    // Date
    let date_size = if config.clock.show_date && !compact { time_size * 0.25 } else { 0.0 };
    let date_w = if date_size > 0.0 {
        let sample = chrono::Local::now().format(&config.clock.date_format).to_string();
        font.measure_text(&sample, date_size).0
    } else {
        0.0
    };
    let date_gap = if date_size > 0.0 { time_size * 0.15 } else { 0.0 };

    // Battery
    let battery_h = if config.battery.enabled { time_size * 0.35 } else { 0.0 };
    let battery_gap = if battery_h > 0.0 { pad_y * 0.5 } else { 0.0 };

    // Subclocks
    let (subclock_w, subclock_h) = compute_subclock_size(config, font, time_size, pad_y);

    let width = time_w.max(date_w).max(subclock_w) + pad_x * 2.0;
    let height = pad_y + battery_h + battery_gap + time_size + date_gap + date_size + subclock_h + pad_y;

    (width.ceil() as u32, height.ceil() as u32)
}

fn compute_analogue_size(config: &ClockConfig, font: &FontState, compact: bool) -> (u32, u32) {
    let diameter = config.clock.diameter as f32;
    let effective = if compact { diameter * 0.75 } else { diameter };
    let pad = 12.0;

    let base = diameter * 0.25;
    let pad_y = base * 0.25;
    let (subclock_w, subclock_h) = compute_subclock_size(config, font, base, pad_y);

    let width = effective.max(subclock_w) + pad * 2.0;
    let height = effective + subclock_h + pad * 2.0;

    (width.ceil() as u32, height.ceil() as u32)
}

fn compute_subclock_size(config: &ClockConfig, font: &FontState, base: f32, _pad_y: f32) -> (f32, f32) {
    let tz_count = config.timezone.len().min(2);
    if tz_count == 0 {
        return (0.0, 0.0);
    }

    let sz = SubclockSizing::from_base(base);

    // Measure widest subclock column
    let widest_sc_time = widest_time_string(config);
    let (sc_time_w, _) = font.measure_text(&widest_sc_time, sz.time_size);
    // Also consider label widths
    let max_label_w = config.timezone.iter().take(2)
        .map(|tz| font.measure_text(&tz.label, sz.label_size).0)
        .fold(0.0f32, f32::max);
    let sc_col_w = sc_time_w.max(max_label_w) + base * 0.2;
    let subclock_w = sc_col_w * tz_count as f32;

    (subclock_w, sz.area_h)
}

fn widest_time_string(config: &ClockConfig) -> String {
    let time_part = if config.clock.show_seconds { "00:00:00" } else { "00:00" };
    let suffix = if config.clock.hour_format == 12 { " PM" } else { "" };
    format!("{}{}", time_part, suffix)
}

/// Render just the background layer (image/solid fill, face).
pub fn render_background(canvas: &mut Canvas, state: &ClockState, font: &FontState) {
    match state.config.clock.face {
        FaceMode::Digital => digital::render_background(canvas, state, font),
        FaceMode::Analogue => analogue::render_background(canvas, state, font),
    }
}

/// Render the foreground layer (text, hands, battery, subclocks).
pub fn render_foreground(canvas: &mut Canvas, state: &ClockState, font: &FontState) {
    match state.config.clock.face {
        FaceMode::Digital => digital::render_foreground(canvas, state, font),
        FaceMode::Analogue => analogue::render_foreground(canvas, state, font),
    }

    // Draw battery indicator
    if state.config.battery.enabled {
        if let Some(ref info) = state.battery {
            battery::render(canvas, state, font, info);
        }
    }

    // Draw subclocks
    if !state.config.timezone.is_empty() {
        subclock::render(canvas, state, font);
    }
}
