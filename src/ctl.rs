use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use crate::ipc;

#[derive(Parser, Debug)]
#[command(name = "ctl", about = "Control a running clockie instance")]
pub struct CtlArgs {
    /// Override socket path
    #[arg(long)]
    socket: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Set or toggle clock face mode
    Face {
        /// digital, analogue, or toggle
        mode: String,
    },
    /// Control compact mode
    Compact {
        /// on, off, or toggle
        mode: String,
    },
    /// Set font size (digital) or diameter (analogue), or scale by +/-N
    Size {
        args: Vec<String>,
    },
    /// Reload configuration file
    Reload,
    /// Print current state as JSON
    State,
    /// Control drag lock
    Lock {
        /// on, off, or toggle
        mode: String,
    },
    /// Move clock to a specific output (monitor name, "next", or "prev")
    Output {
        /// Output name (e.g. HDMI-A-1), or "next"/"prev" to cycle
        name: String,
    },
    /// Control face/background image gallery
    Gallery {
        #[command(subcommand)]
        action: GalleryAction,
    },
    /// Shut down clockie
    Quit,
    /// Generate shell completions for the ctl subcommand
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

#[derive(Subcommand, Debug)]
enum GalleryAction {
    /// Advance to the next gallery image
    Next,
    /// Go back to the previous gallery image
    Prev,
    /// Jump to a specific gallery image by index
    Set {
        /// Zero-based image index
        index: usize,
    },
    /// Start auto-rotating gallery images
    Start {
        /// Rotation interval in seconds (uses configured value if omitted)
        #[arg(long)]
        interval: Option<u64>,
    },
    /// Stop auto-rotating gallery images
    Stop,
    /// Set the auto-rotate interval in seconds
    Interval {
        /// Interval in seconds
        seconds: u64,
    },
}

fn send_command(socket: &PathBuf, cmd: serde_json::Value) -> Result<serde_json::Value> {
    let mut stream = UnixStream::connect(socket)
        .with_context(|| format!("Failed to connect to clockie at {}", socket.display()))?;

    let msg = serde_json::to_string(&cmd)? + "\n";
    stream.write_all(msg.as_bytes())?;
    stream.flush()?;

    let mut reader = BufReader::new(&stream);
    let mut response = String::new();
    reader.read_line(&mut response)?;

    let resp: serde_json::Value = serde_json::from_str(&response)
        .context("Failed to parse response from clockie")?;
    Ok(resp)
}

pub fn run(args: CtlArgs) -> Result<()> {
    // Handle completions before connecting to socket
    if let Commands::Completions { shell } = &args.command {
        let mut cmd = crate::Cli::command();
        clap_complete::generate(*shell, &mut cmd, "clockie", &mut std::io::stdout());
        return Ok(());
    }

    let sock = ipc::socket_path(args.socket.as_ref());

    let cmd = match &args.command {
        Commands::Face { mode } => match mode.as_str() {
            "digital" => json!({"cmd": "set-face", "face": "digital"}),
            "analogue" => json!({"cmd": "set-face", "face": "analogue"}),
            "toggle" => json!({"cmd": "toggle-face"}),
            other => anyhow::bail!("Unknown face mode: {}. Use digital, analogue, or toggle", other),
        },
        Commands::Compact { mode } => match mode.as_str() {
            "on" => json!({"cmd": "set-compact", "compact": true}),
            "off" => json!({"cmd": "set-compact", "compact": false}),
            "toggle" => json!({"cmd": "toggle-compact"}),
            other => anyhow::bail!("Unknown compact mode: {}. Use on, off, or toggle", other),
        },
        Commands::Size { args } => {
            if args.len() == 1 {
                let s = &args[0];
                if s.starts_with('+') || s.starts_with('-') {
                    let delta: i32 = s.parse().context("Invalid delta")?;
                    json!({"cmd": "scale-by", "delta": delta})
                } else {
                    if let Ok(size) = s.parse::<f32>() {
                        json!({"cmd": "set-font-size", "size": size})
                    } else {
                        anyhow::bail!("Invalid size value: {}", s);
                    }
                }
            } else if args.len() == 2 {
                match args[0].as_str() {
                    "font" => {
                        let size: f32 = args[1].parse().context("Invalid font size")?;
                        json!({"cmd": "set-font-size", "size": size})
                    }
                    "diameter" => {
                        let d: u32 = args[1].parse().context("Invalid diameter")?;
                        json!({"cmd": "set-diameter", "diameter": d})
                    }
                    _ => anyhow::bail!("Size requires: <value>, +/-N, font <size>, or diameter <px>"),
                }
            } else {
                anyhow::bail!("Size requires: <value>, +/-N, font <size>, or diameter <px>");
            }
        },
        Commands::Lock { mode } => match mode.as_str() {
            "on" => json!({"cmd": "set-locked", "locked": true}),
            "off" => json!({"cmd": "set-locked", "locked": false}),
            "toggle" => json!({"cmd": "toggle-locked"}),
            other => anyhow::bail!("Unknown lock mode: {}. Use on, off, or toggle", other),
        },
        Commands::Gallery { action } => match action {
            GalleryAction::Next => json!({"cmd": "gallery-next"}),
            GalleryAction::Prev => json!({"cmd": "gallery-prev"}),
            GalleryAction::Set { index } => json!({"cmd": "gallery-set", "index": index}),
            GalleryAction::Start { interval } => {
                let mut cmd = json!({"cmd": "gallery-rotate-start"});
                if let Some(secs) = interval {
                    cmd["interval"] = json!(secs);
                }
                cmd
            }
            GalleryAction::Stop => json!({"cmd": "gallery-rotate-stop"}),
            GalleryAction::Interval { seconds } => json!({"cmd": "gallery-rotate-interval", "seconds": seconds}),
        },
        Commands::Output { name } => json!({"cmd": "move-to-output", "name": name}),
        Commands::Reload => json!({"cmd": "reload-config"}),
        Commands::State => json!({"cmd": "get-state"}),
        Commands::Quit => json!({"cmd": "quit"}),
        Commands::Completions { .. } => unreachable!("handled above"),
    };

    let resp = send_command(&sock, cmd)?;

    if let Some(true) = resp.get("ok").and_then(|v| v.as_bool()) {
        if matches!(&args.command, Commands::State) {
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
    } else {
        let err = resp.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error");
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }

    Ok(())
}
