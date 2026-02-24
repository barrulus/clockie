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
    pub timezone: Vec<TimezoneEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
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
    #[serde(default = "default_true")]
    pub resizable: bool,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default)]
    pub compact: bool,
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
pub struct TimezoneEntry {
    pub label: String,
    pub tz: String,
}

// Defaults

fn default_width() -> u32 { 320 }
fn default_height() -> u32 { 240 }
fn default_layer() -> String { "top".into() }
fn default_anchor() -> String { "top right".into() }
fn default_margin() -> i32 { 20 }
fn default_true() -> bool { true }
fn default_opacity() -> f32 { 1.0 }
fn default_face() -> FaceMode { FaceMode::Digital }
fn default_hour_format() -> u8 { 12 }
fn default_date_format() -> String { "%A, %d %B %Y".into() }
fn default_font() -> String { "monospace".into() }
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
            timezone: Vec::new(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            layer: default_layer(),
            anchor: default_anchor(),
            margin_top: default_margin(),
            margin_bottom: 0,
            margin_left: 0,
            margin_right: default_margin(),
            resizable: true,
            opacity: default_opacity(),
            compact: false,
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

pub fn load_config(path: &std::path::Path) -> Result<ClockConfig> {
    if !path.exists() {
        log::info!("Config file not found at {}, using defaults", path.display());
        return Ok(ClockConfig::default());
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config: {}", path.display()))?;
    let config: ClockConfig = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config: {}", path.display()))?;
    Ok(config)
}
