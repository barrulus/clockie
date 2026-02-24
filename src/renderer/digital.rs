use crate::canvas::{self, Canvas, FontState};
use crate::renderer::ClockState;

pub fn render(canvas: &mut Canvas, state: &ClockState, font: &FontState) {
    let w = canvas.width() as f32;
    let h = canvas.height() as f32;
    let config = &state.config;
    let theme = &config.theme;

    // Draw background image or solid color
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

    let compact = state.compact;
    let padding = w * 0.05; // 5% horizontal padding

    // Calculate timezone row count for layout
    let tz_count = config.timezone.len().min(2);
    let tz_area_ratio = if tz_count > 0 { 0.2 } else { 0.0 };

    // Calculate available height for main clock area
    let clock_area_h = h * (1.0 - tz_area_ratio);
    let available_w = w - padding * 2.0;

    // Time text
    let time_str = state.time.format_time(config.clock.hour_format, config.clock.show_seconds);
    let suffix = state.time.format_time_suffix(config.clock.hour_format);
    let full_time = format!("{}{}", time_str, suffix);

    // Start with height-based size, then constrain to width
    let time_size_by_h = if compact {
        clock_area_h * 0.35
    } else {
        clock_area_h * 0.45
    };
    let time_size = fit_text_size(font, &full_time, time_size_by_h, available_w);

    // Measure and center time text
    let (tw, _) = font.measure_text(&full_time, time_size);
    let time_x = (w - tw) / 2.0;

    let time_y = if config.clock.show_date && !compact {
        clock_area_h * 0.2
    } else {
        (clock_area_h - time_size) / 2.0
    };

    font.draw_text(canvas, &full_time, time_x, time_y, time_size, theme.fg_color);

    // Date string
    if config.clock.show_date && !compact {
        let date_size_target = time_size * 0.25;
        let date_size = fit_text_size(font, &state.time.date_string, date_size_target, available_w);
        let (dw, _) = font.measure_text(&state.time.date_string, date_size);
        let date_x = (w - dw) / 2.0;
        let date_y = time_y + time_size * 1.15;
        font.draw_text(canvas, &state.time.date_string, date_x, date_y, date_size, theme.fg_color);
    }
}

/// Scale font size down until text fits within max_width
fn fit_text_size(font: &FontState, text: &str, initial_size: f32, max_width: f32) -> f32 {
    let size = initial_size.max(10.0);
    let (tw, _) = font.measure_text(text, size);
    if tw <= max_width {
        return size;
    }
    // Scale proportionally
    (size * max_width / tw).max(10.0)
}
