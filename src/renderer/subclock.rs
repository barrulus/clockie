use crate::canvas::{Canvas, FontState};
use crate::renderer::ClockState;
use crate::time_utils;

pub fn render(canvas: &mut Canvas, state: &ClockState, font: &FontState) {
    let w = canvas.width() as f32;
    let h = canvas.height() as f32;
    let config = &state.config;
    let theme = &config.theme;

    let timezones: Vec<_> = config.timezone.iter().take(2).collect();
    if timezones.is_empty() { return; }

    let tz_area_h = h * 0.2;
    let tz_y_start = h - tz_area_h;

    // Draw separator line
    canvas.draw_line(w * 0.1, tz_y_start, w * 0.9, tz_y_start, theme.fg_color, 1.0);

    let row_h = tz_area_h / timezones.len() as f32;
    let label_size = if state.compact { h * 0.06 } else { h * 0.08 };
    let label_size = label_size.max(10.0).min(24.0);
    let time_size = label_size * 1.2;

    for (i, tz) in timezones.iter().enumerate() {
        let row_y = tz_y_start + row_h * i as f32 + row_h * 0.3;

        let time_str = time_utils::timezone_time(
            &tz.tz,
            config.clock.hour_format,
            config.clock.show_seconds,
        ).unwrap_or_else(|| "??:??".into());

        let label_text = format!("{}  ", tz.label);
        let (lw, _) = font.measure_text(&label_text, label_size);
        let (tw, _) = font.measure_text(&time_str, time_size);

        let total_w = lw + tw;
        let start_x = (w - total_w) / 2.0;

        // Muted color for label
        let label_color = [theme.fg_color[0], theme.fg_color[1], theme.fg_color[2], 0xAA];
        font.draw_text(canvas, &label_text, start_x, row_y, label_size, label_color);
        font.draw_text(canvas, &time_str, start_x + lw, row_y, time_size, theme.fg_color);
    }
}
