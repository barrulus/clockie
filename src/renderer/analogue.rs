use crate::canvas::{self, Canvas, FontState};
use crate::config::{AnalogueConfig, HandCap, NumeralStyle, TickStyle, TickVisibility};
use crate::renderer::{draw_contrast_text, ClockState, ContrastInfo, SubclockSizing};

/// Render the analogue clock background: clear + face image or procedural face.
pub fn render_background(canvas: &mut Canvas, state: &ClockState, font: &FontState) {
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
            draw_procedural_face(canvas, font, cx, cy, radius, &config.analogue, &config.theme, &state.contrast);
        }
    } else {
        draw_procedural_face(canvas, font, cx, cy, radius, &config.analogue, &config.theme, &state.contrast);
    }
}

/// Render the analogue clock foreground: hands and centre boss.
pub fn render_foreground(canvas: &mut Canvas, state: &ClockState, _font: &FontState) {
    let w = canvas.width() as f32;
    let h = canvas.height() as f32;
    let config = &state.config;
    let theme = &config.theme;
    let acfg = &config.analogue;

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
    draw_hand(canvas, cx, cy, hr_angle,
        radius * acfg.hour_hand_length * hand_scale,
        radius * acfg.hour_hand_width,
        theme.hour_hand_color, acfg);
    // Minute hand
    draw_hand(canvas, cx, cy, min_angle,
        radius * acfg.minute_hand_length * hand_scale,
        radius * acfg.minute_hand_width,
        theme.minute_hand_color, acfg);
    // Second hand
    draw_hand(canvas, cx, cy, sec_angle,
        radius * acfg.second_hand_length * hand_scale,
        radius * acfg.second_hand_width,
        theme.second_hand_color, acfg);

    // Centre boss
    canvas.draw_circle(cx, cy, radius * 0.05, state.contrast.text_color, true, 0.0);
}

fn draw_procedural_face(
    canvas: &mut Canvas,
    font: &FontState,
    cx: f32, cy: f32, radius: f32,
    acfg: &AnalogueConfig,
    theme: &crate::config::ThemeConfig,
    contrast: &ContrastInfo,
) {
    // 1. Face fill
    if let Some(fill) = acfg.face_fill {
        canvas.draw_circle(cx, cy, radius, fill, true, 0.0);
    }

    // 2. Bezel
    if acfg.bezel_width > 0.0 {
        let stroke_w = radius * acfg.bezel_width;
        canvas.draw_circle(cx, cy, radius, acfg.bezel_color, false, stroke_w);
    } else {
        // Default thin 2px stroke (current behavior)
        canvas.draw_circle(cx, cy, radius, theme.tick_color, false, 2.0);
    }

    // 3. Minute track
    if acfg.minute_track_width > 0.0 {
        let track_r = radius * 0.92;
        let stroke_w = radius * acfg.minute_track_width;
        canvas.draw_circle(cx, cy, track_r, acfg.minute_track_color, false, stroke_w);
    }

    // 4. Ticks
    draw_ticks(canvas, cx, cy, radius, acfg, theme);

    // 5. Numerals
    draw_numerals(canvas, font, cx, cy, radius, acfg, contrast);
}

fn draw_ticks(
    canvas: &mut Canvas,
    cx: f32, cy: f32, radius: f32,
    acfg: &AnalogueConfig,
    theme: &crate::config::ThemeConfig,
) {
    if acfg.show_ticks == TickVisibility::None {
        return;
    }

    for i in 0..60 {
        let is_hour = i % 5 == 0;
        let is_quarter = i % 15 == 0;

        let should_draw = match acfg.show_ticks {
            TickVisibility::All60 => true,
            TickVisibility::HoursOnly => is_hour,
            TickVisibility::QuartersOnly => is_quarter,
            TickVisibility::None => false,
        };
        if !should_draw { continue; }

        let angle = (i as f32 * 6.0 - 90.0).to_radians();
        let inner = if is_hour { radius * 0.85 } else { radius * 0.92 };
        let outer = radius * 0.98;

        match acfg.tick_style {
            TickStyle::Line => {
                let tick_width = if is_hour { 2.5 } else { 1.0 };
                let x1 = cx + inner * angle.cos();
                let y1 = cy + inner * angle.sin();
                let x2 = cx + outer * angle.cos();
                let y2 = cy + outer * angle.sin();
                canvas.draw_line(x1, y1, x2, y2, theme.tick_color, tick_width);
            }
            TickStyle::Dot => {
                let mid = (inner + outer) / 2.0;
                let dot_cx = cx + mid * angle.cos();
                let dot_cy = cy + mid * angle.sin();
                let dot_r = if is_hour { 3.0 } else { 1.5 };
                canvas.draw_circle(dot_cx, dot_cy, dot_r, theme.tick_color, true, 0.0);
            }
            TickStyle::Diamond => {
                let mid = (inner + outer) / 2.0;
                let half_len = (outer - inner) / 2.0;
                let half_w = if is_hour { 2.5 } else { 1.2 };
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                // Diamond: 4 points along and perpendicular to the radial axis
                let points = [
                    (cx + (mid + half_len) * cos_a, cy + (mid + half_len) * sin_a), // outer tip
                    (cx + mid * cos_a - half_w * sin_a, cy + mid * sin_a + half_w * cos_a), // left
                    (cx + (mid - half_len) * cos_a, cy + (mid - half_len) * sin_a), // inner tip
                    (cx + mid * cos_a + half_w * sin_a, cy + mid * sin_a - half_w * cos_a), // right
                ];
                canvas.fill_polygon(&points, theme.tick_color);
            }
        }
    }
}

fn draw_numerals(
    canvas: &mut Canvas,
    font: &FontState,
    cx: f32, cy: f32, radius: f32,
    acfg: &AnalogueConfig,
    contrast: &ContrastInfo,
) {
    let labels: &[&str] = match acfg.numerals {
        NumeralStyle::None => return,
        NumeralStyle::Arabic => &["12", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11"],
        NumeralStyle::Roman => &["XII", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI"],
    };

    let text_size = radius * acfg.numeral_size;
    let dist = radius * (1.0 - acfg.numeral_inset);

    for (i, label) in labels.iter().enumerate() {
        let angle = (i as f32 * 30.0 - 90.0).to_radians();
        let nx = cx + dist * angle.cos();
        let ny = cy + dist * angle.sin();

        let (tw, th) = font.measure_text(label, text_size);
        let tx = nx - tw / 2.0;
        let ty = ny - th / 2.0;

        draw_contrast_text(font, canvas, label, tx, ty, text_size, contrast.text_color, contrast);
    }
}

fn draw_hand(
    canvas: &mut Canvas,
    cx: f32, cy: f32,
    angle_deg: f32, length: f32, width: f32,
    color: [u8; 4],
    acfg: &AnalogueConfig,
) {
    let angle = (angle_deg - 90.0).to_radians();
    let cos_a = angle.cos();
    let sin_a = angle.sin();

    // Shadow pass
    if acfg.hand_shadow {
        let shadow_color = [0x00, 0x00, 0x00, 0x60];
        let sx = 2.0;
        let sy = 2.0;
        draw_hand_shape(canvas, cx + sx, cy + sy, cos_a, sin_a, length, width, shadow_color, acfg);
    }

    // Main hand
    draw_hand_shape(canvas, cx, cy, cos_a, sin_a, length, width, color, acfg);
}

fn draw_hand_shape(
    canvas: &mut Canvas,
    cx: f32, cy: f32,
    cos_a: f32, sin_a: f32,
    length: f32, width: f32,
    color: [u8; 4],
    acfg: &AnalogueConfig,
) {
    let half_w = width / 2.0;

    match acfg.hand_cap {
        HandCap::Arrow => {
            // Narrow shaft (50% width, 80% length) + triangle arrowhead
            let shaft_len = length * 0.8;
            let shaft_half_w = half_w * 0.5;

            // Shaft as a thin rectangle
            let sx = cx + shaft_len * cos_a;
            let sy = cy + shaft_len * sin_a;
            let shaft = [
                (cx - shaft_half_w * sin_a, cy + shaft_half_w * cos_a),
                (cx + shaft_half_w * sin_a, cy - shaft_half_w * cos_a),
                (sx + shaft_half_w * sin_a, sy - shaft_half_w * cos_a),
                (sx - shaft_half_w * sin_a, sy + shaft_half_w * cos_a),
            ];
            canvas.fill_polygon(&shaft, color);

            // Arrowhead triangle
            let tip_x = cx + length * cos_a;
            let tip_y = cy + length * sin_a;
            let arrow = [
                (tip_x, tip_y),
                (sx - half_w * sin_a, sy + half_w * cos_a),
                (sx + half_w * sin_a, sy - half_w * cos_a),
            ];
            canvas.fill_polygon(&arrow, color);
        }
        HandCap::Round | HandCap::Flat => {
            if acfg.hand_taper > 0.0 {
                // Tapered trapezoid: full width at base, narrower at tip
                let tip_half_w = half_w * (1.0 - acfg.hand_taper.clamp(0.0, 1.0));
                let tip_x = cx + length * cos_a;
                let tip_y = cy + length * sin_a;
                let points = [
                    (cx - half_w * sin_a, cy + half_w * cos_a),
                    (cx + half_w * sin_a, cy - half_w * cos_a),
                    (tip_x + tip_half_w * sin_a, tip_y - tip_half_w * cos_a),
                    (tip_x - tip_half_w * sin_a, tip_y + tip_half_w * cos_a),
                ];
                canvas.fill_polygon(&points, color);
            } else {
                // Simple line (unchanged visual — current default)
                let x2 = cx + length * cos_a;
                let y2 = cy + length * sin_a;
                canvas.draw_line(cx, cy, x2, y2, color, width);
            }
        }
    }
}
