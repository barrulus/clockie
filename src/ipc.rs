use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "kebab-case")]
pub enum IpcCommand {
    SetFace { face: String },
    ToggleFace,
    SetCompact { compact: bool },
    ToggleCompact,
    SetFontSize { size: f32 },
    SetDiameter { diameter: u32 },
    ScaleBy { delta: i32 },
    SetLocked { locked: bool },
    ToggleLocked,
    MoveToOutput { name: String },
    ReloadConfig,
    GetState,
    Quit,
    GalleryNext,
    GalleryPrev,
    GallerySet { index: usize },
    GalleryRotateStart { interval: Option<u64> },
    GalleryRotateStop,
    GalleryRotateInterval { seconds: u64 },
}

#[derive(Debug, Serialize)]
pub struct IpcResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    // State fields (only for get-state)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub face: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compact: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diameter: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gallery_digital_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gallery_analogue_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gallery_digital_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gallery_analogue_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gallery_rotate_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gallery_rotate_interval: Option<u64>,
}

impl IpcResponse {
    pub fn ok() -> Self {
        Self {
            ok: true, error: None, face: None, compact: None, width: None,
            height: None, font_size: None, diameter: None, config_path: None,
            locked: None, output: None, gallery_digital_index: None,
            gallery_analogue_index: None, gallery_digital_count: None,
            gallery_analogue_count: None, gallery_rotate_active: None,
            gallery_rotate_interval: None,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            ok: false, error: Some(msg.into()), face: None, compact: None,
            width: None, height: None, font_size: None, diameter: None,
            config_path: None, locked: None, output: None,
            gallery_digital_index: None, gallery_analogue_index: None,
            gallery_digital_count: None, gallery_analogue_count: None,
            gallery_rotate_active: None, gallery_rotate_interval: None,
        }
    }

    pub fn state(face: &str, compact: bool, width: u32, height: u32, font_size: f32, diameter: u32, config_path: &str, locked: bool, output: Option<&str>) -> Self {
        Self {
            ok: true,
            error: None,
            face: Some(face.into()),
            compact: Some(compact),
            width: Some(width),
            height: Some(height),
            font_size: Some(font_size),
            diameter: Some(diameter),
            config_path: Some(config_path.into()),
            locked: Some(locked),
            output: output.map(|s| s.into()),
            gallery_digital_index: None,
            gallery_analogue_index: None,
            gallery_digital_count: None,
            gallery_analogue_count: None,
            gallery_rotate_active: None,
            gallery_rotate_interval: None,
        }
    }

    pub fn with_gallery(mut self, digital_index: usize, analogue_index: usize, digital_count: usize, analogue_count: usize, rotate_active: bool, rotate_interval: u64) -> Self {
        self.gallery_digital_index = Some(digital_index);
        self.gallery_analogue_index = Some(analogue_index);
        self.gallery_digital_count = Some(digital_count);
        self.gallery_analogue_count = Some(analogue_count);
        self.gallery_rotate_active = Some(rotate_active);
        self.gallery_rotate_interval = Some(rotate_interval);
        self
    }
}

pub fn socket_path(override_path: Option<&PathBuf>) -> PathBuf {
    if let Some(p) = override_path {
        return p.clone();
    }
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(dir).join("clockie.sock")
    } else {
        let uid = unsafe { libc::getuid() };
        PathBuf::from(format!("/tmp/clockie-{}.sock", uid))
    }
}

pub fn create_listener(path: &PathBuf) -> Result<UnixListener> {
    // Remove stale socket
    if path.exists() {
        // Check if another instance is running
        if UnixStream::connect(path).is_ok() {
            anyhow::bail!("Another clockie instance is already running (socket {} is active)", path.display());
        }
        std::fs::remove_file(path)?;
    }

    let listener = UnixListener::bind(path)?;
    listener.set_nonblocking(true)?;
    log::info!("IPC listening on {}", path.display());
    Ok(listener)
}

pub fn cleanup_socket(path: &PathBuf) {
    if path.exists() {
        let _ = std::fs::remove_file(path);
        log::info!("Removed socket {}", path.display());
    }
}

pub fn read_command(stream: &UnixStream) -> Result<IpcCommand> {
    let reader = BufReader::new(stream);
    let mut line = String::new();
    let mut reader = reader;
    reader.read_line(&mut line)?;
    let cmd: IpcCommand = serde_json::from_str(line.trim())?;
    Ok(cmd)
}

pub fn write_response(stream: &mut UnixStream, response: &IpcResponse) -> Result<()> {
    let json = serde_json::to_string(response)?;
    stream.write_all(json.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;
    Ok(())
}
