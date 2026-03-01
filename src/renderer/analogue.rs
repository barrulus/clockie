use crate::canvas::{self, Canvas, FontState};
use crate::renderer::{ClockState, SubclockSizing};

/// Render the analogue clock background: clear + face image or procedural face.
pub fn render_background(canvas: &mut Canvas, state: &ClockState, _font: &FontState) {
    let w = canvas.width() as f32;
    let h = canvas.height() as f32;
    let config = &state.config;
    let theme = &config.theme;

    // Clear background
    canvas.clear(theme.bg_color);

    let diameter = config.clock.diameter as f32;
    let effective = if state.compact { diameter * 0.75 } else { diameter };
    let radius = effective / 2.0;

    // Subclock area height (hidden in compact mode)
    let subclock_h = if !state.compact && !config.timezone.is_empty() {
        let tz_count = config.timezone.len().min(2);
        let sz = SubclockSizing::from_base(diameter * 0.25);
        sz.area_h * tz_count as f32
    } else {
        0.0
    };

    // Clock area is total height minus subclock area
    let clock_area_h = h - subclock_h;
    let cx = w / 2.0;
    let cy = clock_area_h / 2.0;

    // Draw face image or procedural face
    if !config.background.analogue_face_image.is_empty() {
        let path = &config.background.analogue_face_image;
        let size = (radius * 2.0) as u32;
        let face = if canvas::is_svg(path) {
            canvas::load_svg(path, size, size)
        } else {
            canvas::load_image(path).map(|img| canvas::scale_image(&img, size, size, &config.background.image_scale))
        };
        if let Some(img) = face {
            canvas.draw_image(&img, (cx - radius) as i32, (cy - radius) as i32);
        } else {
            draw_procedural_face(canvas, cx, cy, radius, theme);
        }
    } else {
        draw_procedural_face(canvas, cx, cy, radius, theme);
    }
}

/// Render the analogue clock foreground: hands and centre boss.
pub fn render_foreground(canvas: &mut Canvas, state: &ClockState, _font: &FontState) {
    let w = canvas.width() as f32;
    let h = canvas.height() as f32;
    let config = &state.config;
    let theme = &config.theme;

    let diameter = config.clock.diameter as f32;
    let effective = if state.compact { diameter * 0.75 } else { diameter };
    let radius = effective / 2.0;

    // Subclock area height (hidden in compact mode)
    let subclock_h = if !state.compact && !config.timezone.is_empty() {
        let tz_count = config.timezone.len().min(2);
        let sz = SubclockSizing::from_base(diameter * 0.25);
        sz.area_h * tz_count as f32
    } else {
        0.0
    };

    let clock_area_h = h - subclock_h;
    let cx = w / 2.0;
    let cy = clock_area_h / 2.0;

    // Draw hands
    let sec = state.time.second as f32;
    let min = state.time.minute as f32 + sec / 60.0;
    let hr = (state.time.hour % 12) as f32 + min / 60.0;

    let sec_angle = sec * 6.0;
    let min_angle = min * 6.0;
    let hr_angle = hr * 30.0;

    let hand_scale = if state.compact { 0.8 } else { 1.0 };

    // Hour hand
    draw_hand(canvas, cx, cy, hr_angle, radius * 0.55 * hand_scale, radius * 0.06, theme.hour_hand_color);
    // Minute hand
    draw_hand(canvas, cx, cy, min_angle, radius * 0.75 * hand_scale, radius * 0.04, theme.minute_hand_color);
    // Second hand
    draw_hand(canvas, cx, cy, sec_angle, radius * 0.85 * hand_scale, radius * 0.02, theme.second_hand_color);

    // Centre boss
    canvas.draw_circle(cx, cy, radius * 0.05, state.contrast.text_color, true, 0.0);
}

fn draw_procedural_face(canvas: &mut Canvas, cx: f32, cy: f32, radius: f32, theme: &crate::config::ThemeConfig) {
    // Outer circle
    canvas.draw_circle(cx, cy, radius, theme.tick_color, false, 2.0);

    // Tick marks
    for i in 0..60 {
        let angle = (i as f32 * 6.0 - 90.0).to_radians();
        let is_hour = i % 5 == 0;
        let inner = if is_hour { radius * 0.85 } else { radius * 0.92 };
        let outer = radius * 0.98;
        let tick_width = if is_hour { 2.5 } else { 1.0 };

        let x1 = cx + inner * angle.cos();
        let y1 = cy + inner * angle.sin();
        let x2 = cx + outer * angle.cos();
        let y2 = cy + outer * angle.sin();

        canvas.draw_line(x1, y1, x2, y2, theme.tick_color, tick_width);
    }
}

fn draw_hand(canvas: &mut Canvas, cx: f32, cy: f32, angle_deg: f32, length: f32, width: f32, color: [u8; 4]) {
    let angle = (angle_deg - 90.0).to_radians();
    let x2 = cx + length * angle.cos();
    let y2 = cy + length * angle.sin();
    canvas.draw_line(cx, cy, x2, y2, color, width);
}
