mod battery;
mod canvas;
mod config;
mod ctl;
mod ipc;
mod renderer;
mod time_utils;
mod wayland;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "clockie", version, about = "Lightweight Wayland layer-shell desktop clock")]
pub struct Cli {
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

    /// Generate shell completions and exit
    #[arg(long, value_name = "SHELL")]
    completions: Option<Shell>,

    #[command(subcommand)]
    command: Option<CliCommand>,
}

#[derive(Subcommand, Debug)]
enum CliCommand {
    /// Control a running clockie instance
    Ctl(ctl::CtlArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(CliCommand::Ctl(args)) => ctl::run(args),
        None => run_daemon(cli),
    }
}

fn run_daemon(args: Cli) -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    if let Some(shell) = args.completions {
        let mut cmd = Cli::command();
        clap_complete::generate(shell, &mut cmd, "clockie", &mut std::io::stdout());
        return Ok(());
    }

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
    log::info!("Content sizing: font_size={}, diameter={}", config.clock.font_size, config.clock.diameter);

    wayland::run(config, config_path, args.socket)?;

    Ok(())
}
