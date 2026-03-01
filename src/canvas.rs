use tiny_skia::{Color, Paint, PathBuilder, Pixmap, PixmapPaint, Rect, Stroke, Transform};

pub struct Canvas {
    pub pixmap: Pixmap,
}

pub struct FontState {
    font: fontdue::Font,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            pixmap: Pixmap::new(width, height).expect("Failed to create pixmap"),
        }
    }

    pub fn width(&self) -> u32 {
        self.pixmap.width()
    }

    pub fn height(&self) -> u32 {
        self.pixmap.height()
    }

    pub fn clear(&mut self, color: [u8; 4]) {
        self.pixmap.fill(Color::from_rgba8(color[0], color[1], color[2], color[3]));
    }

    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [u8; 4]) {
        if let Some(rect) = Rect::from_xywh(x, y, w, h) {
            let mut paint = Paint::default();
            paint.set_color_rgba8(color[0], color[1], color[2], color[3]);
            paint.anti_alias = true;
            self.pixmap.fill_rect(rect, &paint, Transform::identity(), None);
        }
    }

    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: [u8; 4], width: f32) {
        let mut pb = PathBuilder::new();
        pb.move_to(x1, y1);
        pb.line_to(x2, y2);
        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color_rgba8(color[0], color[1], color[2], color[3]);
            paint.anti_alias = true;
            let stroke = Stroke { width, ..Stroke::default() };
            self.pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }

    pub fn draw_circle(&mut self, cx: f32, cy: f32, r: f32, color: [u8; 4], fill: bool, stroke_width: f32) {
        let mut pb = PathBuilder::new();
        // Approximate circle with 4 cubic bezier curves
        let k = 0.5522847498; // magic constant for cubic bezier circle
        let kr = k * r;
        pb.move_to(cx, cy - r);
        pb.cubic_to(cx + kr, cy - r, cx + r, cy - kr, cx + r, cy);
        pb.cubic_to(cx + r, cy + kr, cx + kr, cy + r, cx, cy + r);
        pb.cubic_to(cx - kr, cy + r, cx - r, cy + kr, cx - r, cy);
        pb.cubic_to(cx - r, cy - kr, cx - kr, cy - r, cx, cy - r);
        pb.close();

        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color_rgba8(color[0], color[1], color[2], color[3]);
            paint.anti_alias = true;
            if fill {
                self.pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
            } else {
                let stroke = Stroke { width: stroke_width, ..Stroke::default() };
                self.pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
        }
    }

    pub fn draw_image(&mut self, img: &Pixmap, x: i32, y: i32) {
        self.pixmap.draw_pixmap(
            x, y, img.as_ref(),
            &PixmapPaint::default(),
            Transform::identity(),
            None,
        );
    }

    #[allow(dead_code)]
    pub fn draw_scaled_image(&mut self, img: &Pixmap, x: f32, y: f32, target_w: f32, target_h: f32) {
        let sx = target_w / img.width() as f32;
        let sy = target_h / img.height() as f32;
        self.pixmap.draw_pixmap(
            0, 0, img.as_ref(),
            &PixmapPaint::default(),
            Transform::from_scale(sx, sy).post_translate(x, y),
            None,
        );
    }

    /// Convert RGBA pixels to BGRA (ARGB8888 in little-endian) for wl_shm
    pub fn pixels_argb8888(&self) -> Vec<u8> {
        let data = self.pixmap.data();
        let mut out = vec![0u8; data.len()];
        for i in (0..data.len()).step_by(4) {
            let r = data[i];
            let g = data[i + 1];
            let b = data[i + 2];
            let a = data[i + 3];
            // BGRA order (ARGB8888 little-endian)
            out[i] = b;
            out[i + 1] = g;
            out[i + 2] = r;
            out[i + 3] = a;
        }
        out
    }
}

impl FontState {
    pub fn new(font_name: &str) -> Self {
        // Try loading as a file path first
        if let Ok(data) = std::fs::read(font_name) {
            if let Ok(font) = fontdue::Font::from_bytes(data, fontdue::FontSettings::default()) {
                return Self { font };
            }
        }

        // Search common system font paths
        let search_paths = [
            "/usr/share/fonts",
            "/usr/local/share/fonts",
            "/nix/var/nix/profiles/system/sw/share/X11/fonts",
        ];

        // Try to find a monospace font
        for base in &search_paths {
            if let Some(font) = Self::search_font_dir(base, font_name) {
                return Self { font };
            }
        }

        // Fallback: try common monospace font files
        let fallback_fonts = [
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
            "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
            "/usr/share/fonts/dejavu-sans-mono-fonts/DejaVuSansMono.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
        ];

        for path in &fallback_fonts {
            if let Ok(data) = std::fs::read(path) {
                if let Ok(font) = fontdue::Font::from_bytes(data, fontdue::FontSettings::default()) {
                    log::info!("Using fallback font: {}", path);
                    return Self { font };
                }
            }
        }

        // Last resort: use built-in minimal font data won't work, so search nix store
        if let Some(font) = Self::search_nix_fonts() {
            return Self { font };
        }

        log::warn!("No system fonts found, text rendering will fail");
        // Create a dummy font from embedded data - we'll use a minimal approach
        Self::with_builtin_fallback()
    }

    fn search_font_dir(dir: &str, _name: &str) -> Option<fontdue::Font> {
        let dir_path = std::path::Path::new(dir);
        if !dir_path.exists() { return None; }

        // Walk directory looking for monospace/dejavu fonts
        Self::walk_for_font(dir_path)
    }

    fn walk_for_font(dir: &std::path::Path) -> Option<fontdue::Font> {
        let entries = std::fs::read_dir(dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(f) = Self::walk_for_font(&path) {
                    return Some(f);
                }
            } else if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if (ext == "ttf" || ext == "otf") && path.to_string_lossy().contains("Mono") {
                    if let Ok(data) = std::fs::read(&path) {
                        if let Ok(font) = fontdue::Font::from_bytes(data, fontdue::FontSettings::default()) {
                            log::info!("Found font: {}", path.display());
                            return Some(font);
                        }
                    }
                }
            }
        }
        None
    }

    fn search_nix_fonts() -> Option<fontdue::Font> {
        // Search /nix/store for font packages
        let nix_store = std::path::Path::new("/nix/store");
        if !nix_store.exists() { return None; }

        if let Ok(entries) = std::fs::read_dir(nix_store) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.contains("dejavu-fonts") || name.contains("liberation-fonts") {
                    if let Some(f) = Self::walk_for_font(&entry.path()) {
                        return Some(f);
                    }
                }
            }
        }
        None
    }

    fn with_builtin_fallback() -> Self {
        // We can't ship a font, but fontdue requires one.
        // As a last resort, create a font from the first .ttf we can find anywhere
        for base in &["/usr/share/fonts", "/nix/store"] {
            if let Some(font) = Self::walk_for_any_font(std::path::Path::new(base)) {
                return Self { font };
            }
        }
        panic!("No fonts found on system. Please install a TTF font or specify a font path in config.");
    }

    fn walk_for_any_font(dir: &std::path::Path) -> Option<fontdue::Font> {
        let entries = std::fs::read_dir(dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(f) = Self::walk_for_any_font(&path) {
                    return Some(f);
                }
            } else if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if ext == "ttf" || ext == "otf" {
                    if let Ok(data) = std::fs::read(&path) {
                        if let Ok(font) = fontdue::Font::from_bytes(data, fontdue::FontSettings::default()) {
                            log::info!("Found fallback font: {}", path.display());
                            return Some(font);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn measure_text(&self, text: &str, size: f32) -> (f32, f32) {
        let mut width = 0.0f32;
        let mut max_height = 0.0f32;
        for ch in text.chars() {
            let metrics = self.font.metrics(ch, size);
            width += metrics.advance_width;
            let h = metrics.height as f32;
            if h > max_height { max_height = h; }
        }
        (width, max_height)
    }

    /// Draw text with a contrasting outline for readability on varied backgrounds.
    /// Draws text at 8 compass offsets in `outline_color`, then the actual text on top.
    pub fn draw_text_outlined(&self, canvas: &mut Canvas, text: &str, x: f32, y: f32, size: f32, color: [u8; 4], outline_color: [u8; 4]) {
        let r = (size * 0.04).max(0.8).min(1.5);
        let offsets: [(f32, f32); 8] = [
            (-r, 0.0), (r, 0.0), (0.0, -r), (0.0, r),
            (-r, -r), (r, -r), (-r, r), (r, r),
        ];
        for (dx, dy) in &offsets {
            self.draw_text(canvas, text, x + dx, y + dy, size, outline_color);
        }
        self.draw_text(canvas, text, x, y, size, color);
    }

    pub fn draw_text(&self, canvas: &mut Canvas, text: &str, x: f32, y: f32, size: f32, color: [u8; 4]) {
        let mut cursor_x = x;
        for ch in text.chars() {
            let (metrics, bitmap) = self.font.rasterize(ch, size);
            if !bitmap.is_empty() && metrics.width > 0 && metrics.height > 0 {
                let gx = cursor_x as i32 + metrics.xmin;
                let gy = y as i32 + size as i32 - metrics.height as i32 - metrics.ymin;
                for row in 0..metrics.height {
                    for col in 0..metrics.width {
                        let coverage = bitmap[row * metrics.width + col];
                        if coverage > 0 {
                            let px = gx + col as i32;
                            let py = gy + row as i32;
                            if px >= 0 && py >= 0 && (px as u32) < canvas.width() && (py as u32) < canvas.height() {
                                let alpha = (coverage as u32 * color[3] as u32) / 255;
                                if alpha > 0 {
                                    blend_pixel(&mut canvas.pixmap, px as u32, py as u32, color, alpha as u8);
                                }
                            }
                        }
                    }
                }
            }
            cursor_x += metrics.advance_width;
        }
    }
}

/// Sample the average perceptual luminance (0–255) of a rectangular region in the canvas.
/// Samples every 4th pixel for performance.
pub fn sample_region_luminance(canvas: &Canvas, x: u32, y: u32, w: u32, h: u32) -> f32 {
    let data = canvas.pixmap.data();
    let cw = canvas.width();
    let ch = canvas.height();
    let x_end = (x + w).min(cw);
    let y_end = (y + h).min(ch);
    let mut sum = 0.0f64;
    let mut count = 0u32;
    let mut py = y;
    while py < y_end {
        let mut px = x;
        while px < x_end {
            let idx = ((py * cw + px) * 4) as usize;
            if idx + 2 < data.len() {
                let r = data[idx] as f64;
                let g = data[idx + 1] as f64;
                let b = data[idx + 2] as f64;
                sum += 0.2126 * r + 0.7152 * g + 0.0722 * b;
                count += 1;
            }
            px += 4;
        }
        py += 4;
    }
    if count == 0 { return 0.0; }
    (sum / count as f64) as f32
}

fn blend_pixel(pixmap: &mut Pixmap, x: u32, y: u32, color: [u8; 4], alpha: u8) {
    let w = pixmap.width();
    let idx = ((y * w + x) * 4) as usize;
    let data = pixmap.data_mut();
    if idx + 3 >= data.len() { return; }

    let a = alpha as u32;
    let inv_a = 255 - a;
    data[idx]     = ((color[0] as u32 * a + data[idx] as u32 * inv_a) / 255) as u8;
    data[idx + 1] = ((color[1] as u32 * a + data[idx + 1] as u32 * inv_a) / 255) as u8;
    data[idx + 2] = ((color[2] as u32 * a + data[idx + 2] as u32 * inv_a) / 255) as u8;
    data[idx + 3] = (a + data[idx + 3] as u32 * inv_a / 255).min(255) as u8;
}

fn expand_tilde(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}/{}", home, rest);
        }
    }
    path.to_string()
}

pub fn is_svg(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".svg") || lower.ends_with(".svgz")
}

pub fn load_svg(path: &str, width: u32, height: u32) -> Option<Pixmap> {
    if path.is_empty() { return None; }
    let expanded = expand_tilde(path);
    let data = std::fs::read(&expanded).ok()?;
    let tree = resvg::usvg::Tree::from_data(&data, &resvg::usvg::Options::default()).ok()?;
    let mut pixmap = Pixmap::new(width, height)?;
    let svg_size = tree.size();
    let sx = width as f32 / svg_size.width();
    let sy = height as f32 / svg_size.height();
    resvg::render(&tree, Transform::from_scale(sx, sy), &mut pixmap.as_mut());
    Some(pixmap)
}

pub fn load_image(path: &str) -> Option<Pixmap> {
    if path.is_empty() { return None; }
    let expanded = expand_tilde(path);
    let img = image::open(&expanded).ok()?.to_rgba8();
    let (w, h) = img.dimensions();
    let mut pixmap = Pixmap::new(w, h)?;
    // image crate gives RGBA, tiny-skia premultiplied RGBA
    let src = img.as_raw();
    let dst = pixmap.data_mut();
    for i in (0..src.len()).step_by(4) {
        let a = src[i + 3] as u32;
        dst[i]     = ((src[i] as u32 * a) / 255) as u8;
        dst[i + 1] = ((src[i + 1] as u32 * a) / 255) as u8;
        dst[i + 2] = ((src[i + 2] as u32 * a) / 255) as u8;
        dst[i + 3] = src[i + 3];
    }
    Some(pixmap)
}

pub fn scale_image(src: &Pixmap, target_w: u32, target_h: u32, mode: &str) -> Pixmap {
    let mut dest = Pixmap::new(target_w, target_h).unwrap();
    let sw = src.width() as f32;
    let sh = src.height() as f32;
    let tw = target_w as f32;
    let th = target_h as f32;

    let (sx, sy, tx, ty) = match mode {
        "fill" => {
            let scale = (tw / sw).max(th / sh);
            let ox = (tw - sw * scale) / 2.0;
            let oy = (th - sh * scale) / 2.0;
            (scale, scale, ox, oy)
        }
        "fit" => {
            let scale = (tw / sw).min(th / sh);
            let ox = (tw - sw * scale) / 2.0;
            let oy = (th - sh * scale) / 2.0;
            (scale, scale, ox, oy)
        }
        "stretch" => (tw / sw, th / sh, 0.0, 0.0),
        "center" => (1.0, 1.0, (tw - sw) / 2.0, (th - sh) / 2.0),
        _ => {
            let scale = (tw / sw).max(th / sh);
            let ox = (tw - sw * scale) / 2.0;
            let oy = (th - sh * scale) / 2.0;
            (scale, scale, ox, oy)
        }
    };

    dest.draw_pixmap(
        0, 0, src.as_ref(),
        &PixmapPaint::default(),
        Transform::from_scale(sx, sy).post_translate(tx, ty),
        None,
    );
    dest
}
