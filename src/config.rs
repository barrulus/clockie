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

pub fn save_margins_to_config(path: &std::path::Path, top: i32, right: i32, bottom: i32, left: i32) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to read config for margin save: {}", e);
            return;
        }
    };

    let mut doc: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(e) => {
            log::warn!("Failed to parse config for margin save: {}", e);
            return;
        }
    };

    if let Some(window) = doc.get_mut("window").and_then(|w| w.as_table_mut()) {
        window.insert("margin_top".into(), toml::Value::Integer(top as i64));
        window.insert("margin_right".into(), toml::Value::Integer(right as i64));
        window.insert("margin_bottom".into(), toml::Value::Integer(bottom as i64));
        window.insert("margin_left".into(), toml::Value::Integer(left as i64));
    } else {
        // No [window] table — create one with just margins
        let mut window = toml::map::Map::new();
        window.insert("margin_top".into(), toml::Value::Integer(top as i64));
        window.insert("margin_right".into(), toml::Value::Integer(right as i64));
        window.insert("margin_bottom".into(), toml::Value::Integer(bottom as i64));
        window.insert("margin_left".into(), toml::Value::Integer(left as i64));
        if let Some(root) = doc.as_table_mut() {
            root.insert("window".into(), toml::Value::Table(window));
        }
    }

    match toml::to_string_pretty(&doc) {
        Ok(output) => {
            if let Err(e) = std::fs::write(path, output) {
                log::warn!("Failed to write config margins: {}", e);
            } else {
                log::info!("Persisted margins to {}", path.display());
            }
        }
        Err(e) => log::warn!("Failed to serialize config: {}", e),
    }
}

pub fn save_output_to_config(path: &std::path::Path, output_name: &str) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to read config for output save: {}", e);
            return;
        }
    };

    let mut doc: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(e) => {
            log::warn!("Failed to parse config for output save: {}", e);
            return;
        }
    };

    if let Some(window) = doc.get_mut("window").and_then(|w| w.as_table_mut()) {
        window.insert("output".into(), toml::Value::String(output_name.into()));
    } else {
        let mut window = toml::map::Map::new();
        window.insert("output".into(), toml::Value::String(output_name.into()));
        if let Some(root) = doc.as_table_mut() {
            root.insert("window".into(), toml::Value::Table(window));
        }
    }

    match toml::to_string_pretty(&doc) {
        Ok(output) => {
            if let Err(e) = std::fs::write(path, output) {
                log::warn!("Failed to write config output: {}", e);
            } else {
                log::info!("Persisted output to {}", path.display());
            }
        }
        Err(e) => log::warn!("Failed to serialize config: {}", e),
    }
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
