use crate::canvas::{self, Canvas, FontState};
use crate::renderer::{ClockState, SubclockSizing, draw_contrast_text};

/// Render the digital clock background: image+scrim or solid fill.
pub fn render_background(canvas: &mut Canvas, state: &ClockState, _font: &FontState) {
    let w = canvas.width() as f32;
    let h = canvas.height() as f32;
    let config = &state.config;
    let theme = &config.theme;

    if !config.background.digital_image.is_empty() {
        if let Some(img) = canvas::load_image(&config.background.digital_image) {
            let scaled = canvas::scale_image(&img, canvas.width(), canvas.height(), &config.background.image_scale);
            canvas.draw_image(&scaled, 0, 0);
            // Apply scrim
            canvas.fill_rect(0.0, 0.0, w, h, theme.bg_color);
        } else {
            canvas.clear(theme.bg_color);
        }
    } else {
        canvas.clear(theme.bg_color);
    }
}

/// Render the digital clock foreground: time text, date text.
pub fn render_foreground(canvas: &mut Canvas, state: &ClockState, font: &FontState) {
    let w = canvas.width() as f32;
    let h = canvas.height() as f32;
    let config = &state.config;

    let compact = state.compact;
    let font_size = config.clock.font_size;
    let time_size = if compact { font_size * 0.7 } else { font_size };
    let pad_y = time_size * 0.25;

    // Time text
    let time_str = state.time.format_time(config.clock.hour_format, config.clock.show_seconds);
    let suffix = state.time.format_time_suffix(config.clock.hour_format);
    let full_time = format!("{}{}", time_str, suffix);

    // Measure and centre time text
    let (tw, _) = font.measure_text(&full_time, time_size);
    let time_x = (w - tw) / 2.0;

    // Date sizing
    let date_size = if config.clock.show_date && !compact { time_size * 0.25 } else { 0.0 };
    let date_gap = if date_size > 0.0 { time_size * 0.15 } else { 0.0 };

    // Battery offset
    let battery_h = if config.battery.enabled { time_size * 0.35 } else { 0.0 };
    let battery_gap = if battery_h > 0.0 { pad_y * 0.5 } else { 0.0 };

    // Subclock area height
    let subclock_h = if !config.timezone.is_empty() {
        SubclockSizing::from_base(time_size).area_h
    } else {
        0.0
    };

    // Clock area is total height minus subclock area
    let clock_area_h = h - subclock_h;

    // Content height within clock area
    let content_h = battery_h + battery_gap + time_size + date_gap + date_size;
    let time_y = (clock_area_h - content_h) / 2.0 + battery_h + battery_gap;

    draw_contrast_text(font, canvas, &full_time, time_x, time_y, time_size, state.contrast.text_color, &state.contrast);

    // Date string
    if config.clock.show_date && !compact {
        let (dw, _) = font.measure_text(&state.time.date_string, date_size);
        let date_x = (w - dw) / 2.0;
        let date_y = time_y + time_size + date_gap;
        draw_contrast_text(font, canvas, &state.time.date_string, date_x, date_y, date_size, state.contrast.text_color, &state.contrast);
    }
}
