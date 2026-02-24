mod canvas;
mod config;
mod ipc;
mod renderer;
mod time_utils;
mod wayland;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "clockie", version, about = "Lightweight Wayland layer-shell desktop clock")]
struct Args {
    /// Path to config file
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Override initial face mode: digital | analogue
    #[arg(long)]
    face: Option<String>,

    /// Start in compact mode
    #[arg(long)]
    compact: bool,

    /// Override first extra timezone
    #[arg(long)]
    tz1: Option<String>,

    /// Override second extra timezone
    #[arg(long)]
    tz2: Option<String>,

    /// Disable timezone sub-clocks
    #[arg(long)]
    no_tz: bool,

    /// Override IPC socket path
    #[arg(long)]
    socket: Option<PathBuf>,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();

    let config_path = args.config.unwrap_or_else(config::default_config_path);
    let mut config = config::load_config(&config_path)?;

    // Apply CLI overrides
    if let Some(face) = &args.face {
        match face.as_str() {
            "digital" => config.clock.face = config::FaceMode::Digital,
            "analogue" => config.clock.face = config::FaceMode::Analogue,
            other => anyhow::bail!("Unknown face mode: {}", other),
        }
    }
    if args.compact {
        config.window.compact = true;
    }
    if args.no_tz {
        config.timezone.clear();
    } else {
        if let Some(tz1) = &args.tz1 {
            if config.timezone.is_empty() {
                config.timezone.push(config::TimezoneEntry { label: tz1.clone(), tz: tz1.clone() });
            } else {
                config.timezone[0] = config::TimezoneEntry { label: tz1.clone(), tz: tz1.clone() };
            }
        }
        if let Some(tz2) = &args.tz2 {
            if config.timezone.len() < 2 {
                config.timezone.push(config::TimezoneEntry { label: tz2.clone(), tz: tz2.clone() });
            } else {
                config.timezone[1] = config::TimezoneEntry { label: tz2.clone(), tz: tz2.clone() };
            }
        }
    }

    // Truncate to max 2 timezone entries
    config.timezone.truncate(2);

    log::info!("Starting clockie with face={:?}, compact={}", config.clock.face, config.window.compact);

    wayland::run(config, config_path, args.socket)?;

    Ok(())
}
