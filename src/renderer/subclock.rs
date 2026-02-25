use crate::canvas::{Canvas, FontState};
use crate::config::FaceMode;
use crate::renderer::ClockState;
use crate::time_utils;

pub fn render(canvas: &mut Canvas, state: &ClockState, font: &FontState) {
    let w = canvas.width() as f32;
    let h = canvas.height() as f32;
    let config = &state.config;
    let theme = &config.theme;

    let timezones: Vec<_> = config.timezone.iter().take(2).collect();
    if timezones.is_empty() { return; }

    // Derive base size from face mode
    let base = match config.clock.face {
        FaceMode::Digital => {
            let font_size = config.clock.font_size;
            if state.compact { font_size * 0.7 } else { font_size }
        }
        FaceMode::Analogue => config.clock.diameter as f32 * 0.25,
    };

    let pad_y = base * 0.25;
    let sc_label_size = base * 0.22;
    let sc_time_size = sc_label_size * 1.3;
    let sc_row_h = sc_label_size + sc_time_size + sc_label_size * 0.1;
    let sc_sep_gap = pad_y * 0.5;
    let tz_area_h = sc_sep_gap + sc_row_h + sc_sep_gap;
    let tz_y_start = h - tz_area_h;

    // Draw separator line
    let sep_color = [theme.fg_color[0], theme.fg_color[1], theme.fg_color[2], 0x66];
    canvas.draw_line(w * 0.05, tz_y_start, w * 0.95, tz_y_start, sep_color, 1.0);

    let tz_count = timezones.len();
    let col_w = w / tz_count as f32;

    // Vertically centre the label+time block within the tz area
    let content_h = sc_label_size + sc_time_size;
    let y_offset = tz_y_start + (tz_area_h - content_h) / 2.0;

    for (i, tz) in timezones.iter().enumerate() {
        let col_cx = col_w * i as f32 + col_w / 2.0;

        let time_str = time_utils::timezone_time(
            &tz.tz,
            config.clock.hour_format,
            config.clock.show_seconds,
        ).unwrap_or_else(|| "??:??".into());

        let label_color = [theme.fg_color[0], theme.fg_color[1], theme.fg_color[2], 0xAA];

        let (lw, _) = font.measure_text(&tz.label, sc_label_size);
        let label_x = col_cx - lw / 2.0;
        font.draw_text(canvas, &tz.label, label_x, y_offset, sc_label_size, label_color);

        let (tw, _) = font.measure_text(&time_str, sc_time_size);
        let time_x = col_cx - tw / 2.0;
        let time_y = y_offset + sc_label_size * 1.1;
        font.draw_text(canvas, &time_str, time_x, time_y, sc_time_size, theme.fg_color);
    }
}
