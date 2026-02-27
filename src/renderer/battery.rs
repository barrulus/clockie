use crate::battery::BatteryInfo;
use crate::canvas::{Canvas, FontState};
use crate::config::FaceMode;
use crate::renderer::{ClockState, draw_contrast_text};

pub fn render(canvas: &mut Canvas, state: &ClockState, font: &FontState, battery: &BatteryInfo) {
    let w = canvas.width() as f32;
    let config = &state.config;

    // Derive icon size from face mode
    let base = match config.clock.face {
        FaceMode::Digital => config.clock.font_size,
        FaceMode::Analogue => config.clock.diameter as f32 * 0.25,
    };
    let icon_h = (base * 0.3).max(12.0);
    let icon_w = icon_h * 1.8;
    let cap_w = icon_w * 0.08; // battery terminal nub width
    let cap_h = icon_h * 0.35;
    let border = (icon_h * 0.08).max(1.5);
    let margin = base * 0.2;

    // Position: top-right corner
    let x = w - icon_w - cap_w - margin;
    let y = margin;

    // Color based on charge level
    let fill_color: [u8; 4] = if battery.percent > 50 {
        [0x4A, 0xDE, 0x80, 0xFF] // green
    } else if battery.percent > 20 {
        [0xFB, 0xBF, 0x24, 0xFF] // yellow
    } else {
        [0xEF, 0x44, 0x44, 0xFF] // red
    };

    let tc = state.contrast.text_color;
    let outline_color: [u8; 4] = [tc[0], tc[1], tc[2], 0xCC];

    // Draw battery outline
    canvas.draw_line(x, y, x + icon_w, y, outline_color, border);
    canvas.draw_line(x, y + icon_h, x + icon_w, y + icon_h, outline_color, border);
    canvas.draw_line(x, y, x, y + icon_h, outline_color, border);
    canvas.draw_line(x + icon_w, y, x + icon_w, y + icon_h, outline_color, border);

    // Terminal nub on right
    let nub_y = y + (icon_h - cap_h) / 2.0;
    canvas.fill_rect(x + icon_w, nub_y, cap_w, cap_h, outline_color);

    // Fill interior based on percentage
    let inner_margin = border + 1.0;
    let inner_x = x + inner_margin;
    let inner_y = y + inner_margin;
    let inner_w = icon_w - inner_margin * 2.0;
    let inner_h = icon_h - inner_margin * 2.0;
    let fill_w = inner_w * (battery.percent as f32 / 100.0);

    if fill_w > 0.0 {
        canvas.fill_rect(inner_x, inner_y, fill_w, inner_h, fill_color);
    }

    // Lightning bolt if charging
    if battery.charging {
        let bolt_color: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
        let cx = x + icon_w / 2.0;
        let cy = y + icon_h / 2.0;
        let bh = icon_h * 0.35;
        let bw = icon_w * 0.12;
        let stroke = (border * 0.8).max(1.0);

        canvas.draw_line(cx + bw * 0.3, cy - bh, cx - bw * 0.5, cy + bh * 0.1, bolt_color, stroke);
        canvas.draw_line(cx - bw * 0.5, cy + bh * 0.1, cx + bw * 0.5, cy - bh * 0.1, bolt_color, stroke);
        canvas.draw_line(cx + bw * 0.5, cy - bh * 0.1, cx - bw * 0.3, cy + bh, bolt_color, stroke);
    }

    // Percentage text to the left of icon
    if state.config.battery.show_percentage {
        let text = format!("{}%", battery.percent);
        let font_size = icon_h * 0.75;
        let (tw, _th) = font.measure_text(&text, font_size);
        let text_x = x - tw - margin * 0.4;
        let text_y = y + (icon_h - font_size) / 2.0;
        let text_color = state.contrast.text_color;
        draw_contrast_text(font, canvas, &text, text_x, text_y, font_size, text_color, &state.contrast);
    }
}
