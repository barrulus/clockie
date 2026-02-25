use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockConfig {
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub clock: ClockSettings,
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub background: BackgroundConfig,
    #[serde(default)]
    pub battery: BatteryConfig,
    #[serde(default)]
    pub timezone: Vec<TimezoneEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_layer")]
    pub layer: String,
    #[serde(default = "default_anchor")]
    pub anchor: String,
    #[serde(default = "default_margin")]
    pub margin_top: i32,
    #[serde(default)]
    pub margin_bottom: i32,
    #[serde(default)]
    pub margin_left: i32,
    #[serde(default = "default_margin")]
    pub margin_right: i32,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default)]
    pub compact: bool,
    #[serde(default)]
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockSettings {
    #[serde(default = "default_face")]
    pub face: FaceMode,
    #[serde(default = "default_hour_format")]
    pub hour_format: u8,
    #[serde(default = "default_true")]
    pub show_seconds: bool,
    #[serde(default = "default_true")]
    pub show_date: bool,
    #[serde(default = "default_date_format")]
    pub date_format: String,
    #[serde(default = "default_font")]
    pub font: String,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default = "default_diameter")]
    pub diameter: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FaceMode {
    Digital,
    Analogue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "default_fg_color", deserialize_with = "deserialize_color")]
    pub fg_color: [u8; 4],
    #[serde(default = "default_bg_color", deserialize_with = "deserialize_color")]
    pub bg_color: [u8; 4],
    #[serde(default = "default_fg_color", deserialize_with = "deserialize_color")]
    pub hour_hand_color: [u8; 4],
    #[serde(default = "default_fg_color", deserialize_with = "deserialize_color")]
    pub minute_hand_color: [u8; 4],
    #[serde(default = "default_second_hand_color", deserialize_with = "deserialize_color")]
    pub second_hand_color: [u8; 4],
    #[serde(default = "default_tick_color", deserialize_with = "deserialize_color")]
    pub tick_color: [u8; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundConfig {
    #[serde(default)]
    pub digital_image: String,
    #[serde(default)]
    pub analogue_face_image: String,
    #[serde(default = "default_image_scale")]
    pub image_scale: String,
    #[serde(default)]
    pub digital_images: Vec<String>,
    #[serde(default)]
    pub analogue_face_images: Vec<String>,
    #[serde(default)]
    pub gallery_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub show_percentage: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimezoneEntry {
    pub label: String,
    pub tz: String,
}

// Defaults

fn default_layer() -> String { "top".into() }
fn default_anchor() -> String { "top right".into() }
fn default_margin() -> i32 { 20 }
fn default_true() -> bool { true }
fn default_opacity() -> f32 { 1.0 }
fn default_face() -> FaceMode { FaceMode::Digital }
fn default_hour_format() -> u8 { 12 }
fn default_date_format() -> String { "%A, %d %B %Y".into() }
fn default_font() -> String { "monospace".into() }
fn default_font_size() -> f32 { 48.0 }
fn default_diameter() -> u32 { 180 }
fn default_image_scale() -> String { "fill".into() }

fn default_fg_color() -> [u8; 4] { [0xFF, 0xFF, 0xFF, 0xFF] }
fn default_bg_color() -> [u8; 4] { [0x00, 0x00, 0x00, 0xCC] }
fn default_second_hand_color() -> [u8; 4] { [0xFF, 0x44, 0x44, 0xFF] }
fn default_tick_color() -> [u8; 4] { [0xCC, 0xCC, 0xCC, 0xFF] }

fn deserialize_color<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 4], D::Error> {
    let s = String::deserialize(d)?;
    parse_color(&s).map_err(serde::de::Error::custom)
}

pub fn parse_color(s: &str) -> Result<[u8; 4]> {
    let s = s.trim_start_matches('#');
    anyhow::ensure!(s.len() == 6 || s.len() == 8, "Color must be RRGGBB or RRGGBBAA");
    let r = u8::from_str_radix(&s[0..2], 16)?;
    let g = u8::from_str_radix(&s[2..4], 16)?;
    let b = u8::from_str_radix(&s[4..6], 16)?;
    let a = if s.len() == 8 { u8::from_str_radix(&s[6..8], 16)? } else { 0xFF };
    Ok([r, g, b, a])
}

// Implementations

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig::default(),
            clock: ClockSettings::default(),
            theme: ThemeConfig::default(),
            background: BackgroundConfig::default(),
            battery: BatteryConfig::default(),
            timezone: Vec::new(),
        }
    }
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            show_percentage: true,
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            layer: default_layer(),
            anchor: default_anchor(),
            margin_top: default_margin(),
            margin_bottom: 0,
            margin_left: 0,
            margin_right: default_margin(),
            opacity: default_opacity(),
            compact: false,
            output: None,
        }
    }
}

impl Default for ClockSettings {
    fn default() -> Self {
        Self {
            face: default_face(),
            hour_format: default_hour_format(),
            show_seconds: true,
            show_date: true,
            date_format: default_date_format(),
            font: default_font(),
            font_size: default_font_size(),
            diameter: default_diameter(),
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            fg_color: default_fg_color(),
            bg_color: default_bg_color(),
            hour_hand_color: default_fg_color(),
            minute_hand_color: default_fg_color(),
            second_hand_color: default_second_hand_color(),
            tick_color: default_tick_color(),
        }
    }
}

impl Default for BackgroundConfig {
    fn default() -> Self {
        Self {
            digital_image: String::new(),
            analogue_face_image: String::new(),
            image_scale: default_image_scale(),
            digital_images: Vec::new(),
            analogue_face_images: Vec::new(),
            gallery_interval: 0,
        }
    }
}

impl BackgroundConfig {
    /// Return the effective list of digital background images.
    /// Uses `digital_images` if non-empty, else wraps `digital_image` into a vec (if non-empty).
    pub fn effective_digital_images(&self) -> Vec<String> {
        if !self.digital_images.is_empty() {
            self.digital_images.clone()
        } else if !self.digital_image.is_empty() {
            vec![self.digital_image.clone()]
        } else {
            Vec::new()
        }
    }

    /// Return the effective list of analogue face images.
    /// Uses `analogue_face_images` if non-empty, else wraps `analogue_face_image` into a vec (if non-empty).
    pub fn effective_analogue_face_images(&self) -> Vec<String> {
        if !self.analogue_face_images.is_empty() {
            self.analogue_face_images.clone()
        } else if !self.analogue_face_image.is_empty() {
            vec![self.analogue_face_image.clone()]
        } else {
            Vec::new()
        }
    }
}

impl FaceMode {
    pub fn toggle(&self) -> Self {
        match self {
            FaceMode::Digital => FaceMode::Analogue,
            FaceMode::Analogue => FaceMode::Digital,
        }
    }
}

pub fn default_config_path() -> PathBuf {
    dirs_path().join("config.toml")
}

fn dirs_path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            PathBuf::from(home).join(".config")
        });
    base.join("clockie")
}

/// Read and parse the config file as a toml_edit document, preserving formatting and comments.
fn read_config_doc(path: &std::path::Path) -> Option<toml_edit::DocumentMut> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to read config: {}", e);
            return None;
        }
    };
    match content.parse::<toml_edit::DocumentMut>() {
        Ok(doc) => Some(doc),
        Err(e) => {
            log::warn!("Failed to parse config: {}", e);
            None
        }
    }
}

/// Write a toml_edit document back to disk, preserving formatting.
fn write_config_doc(path: &std::path::Path, doc: &toml_edit::DocumentMut) {
    if let Err(e) = std::fs::write(path, doc.to_string()) {
        log::warn!("Failed to write config: {}", e);
    }
}

/// Ensure a [window] table exists in the document, creating one if needed.
fn ensure_window_table(doc: &mut toml_edit::DocumentMut) {
    if !doc.contains_key("window") {
        doc["window"] = toml_edit::Item::Table(toml_edit::Table::new());
    }
}

pub fn save_margins_to_config(path: &std::path::Path, top: i32, right: i32, bottom: i32, left: i32) {
    let Some(mut doc) = read_config_doc(path) else { return };
    ensure_window_table(&mut doc);

    doc["window"]["margin_top"] = toml_edit::value(top as i64);
    doc["window"]["margin_right"] = toml_edit::value(right as i64);
    doc["window"]["margin_bottom"] = toml_edit::value(bottom as i64);
    doc["window"]["margin_left"] = toml_edit::value(left as i64);

    write_config_doc(path, &doc);
    log::info!("Persisted margins to {}", path.display());
}

pub fn save_output_to_config(path: &std::path::Path, output_name: &str) {
    let Some(mut doc) = read_config_doc(path) else { return };
    ensure_window_table(&mut doc);

    doc["window"]["output"] = toml_edit::value(output_name);

    write_config_doc(path, &doc);
    log::info!("Persisted output to {}", path.display());
}

pub fn load_config(path: &std::path::Path) -> Result<ClockConfig> {
    if !path.exists() {
        log::info!("Config file not found at {}, generating default", path.display());
        let content = generate_default_config();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::write(path, &content) {
            Ok(()) => log::info!("Created default config at {}", path.display()),
            Err(e) => log::warn!("Failed to write default config: {}", e),
        }
        return Ok(ClockConfig::default());
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config: {}", path.display()))?;
    let config: ClockConfig = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config: {}", path.display()))?;
    Ok(config)
}

fn generate_default_config() -> String {
    r#"# clockie — Wayland layer-shell desktop clock
# Configuration file — generated automatically on first run.
# Uncomment and edit values to customise. Defaults are shown.

[window]
# Layer: background | bottom | top | overlay
layer  = "top"
# Anchor edges: top | bottom | left | right (space-separated)
anchor = "top right"
# Margins from anchored edges (px)
margin_top    = 20
margin_right  = 20
margin_bottom = 0
margin_left   = 0
# Window opacity 0.0–1.0
opacity = 1.0
# Start in compact mode
compact = false
# Output to display on (empty = compositor default)
# output = "HDMI-A-1"

[clock]
# "digital" | "analogue"
face = "digital"
# 12 | 24
hour_format = 12
# Show seconds on digital face
show_seconds = true
# Show date line on digital face
show_date = true
# Date format string (chrono strftime)
date_format = "%A, %d %B %Y"
# Font: system font name or path to .ttf/.otf
font = "monospace"
# Digital mode: main time text size in px (window auto-sizes to fit)
font_size = 48.0
# Analogue mode: clock face diameter in px (window auto-sizes to fit)
diameter = 180

[theme]
# Colours in RRGGBB or RRGGBBAA hex (# prefix optional)
fg_color          = "FFFFFFFF"
bg_color          = "1a1a2eCC"
# Analogue hand colours
hour_hand_color   = "FFFFFFFF"
minute_hand_color = "FFFFFFFF"
second_hand_color = "ef4444FF"
# Tick mark colour (used when no face image)
tick_color        = "CCCCCCFF"

[background]
# Path to a PNG/JPEG behind the digital clock text (empty = bg_color fill)
digital_image = ""
# Path to a PNG/JPEG for the analogue face (replaces drawn ticks)
analogue_face_image = ""
# Scale mode: "fill" | "fit" | "stretch" | "center"
image_scale = "fill"
# Gallery: multiple images to cycle through (overrides single-image fields when non-empty)
# digital_images = ["~/wallpapers/a.png", "~/wallpapers/b.jpg"]
# analogue_face_images = ["~/faces/classic.png", "~/faces/minimal.png"]
# Auto-rotate interval in seconds (0 = disabled)
# gallery_interval = 300

[battery]
# Show a battery indicator in the top-right corner
enabled = false
# Display percentage text next to the icon
show_percentage = true

# Up to 2 timezone sub-clocks. Uncomment to enable.

# [[timezone]]
# label = "London"
# tz    = "Europe/London"

# [[timezone]]
# label = "New York"
# tz    = "America/New_York"
"#.to_string()
}
