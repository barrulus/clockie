use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "clockiectl", version, about = "Control the clockie desktop clock")]
struct Cli {
    /// Override socket path
    #[arg(long)]
    socket: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
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
    /// Set widget size (W H) or resize proportionally (+N / -N)
    Size {
        args: Vec<String>,
    },
    /// Reload configuration file
    Reload,
    /// Print current state as JSON
    State,
    /// Shut down clockie
    Quit,
}

fn socket_path(override_path: Option<&PathBuf>) -> PathBuf {
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

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sock = socket_path(cli.socket.as_ref());

    let cmd = match &cli.command {
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
            if args.len() == 2 {
                let w: u32 = args[0].parse().context("Invalid width")?;
                let h: u32 = args[1].parse().context("Invalid height")?;
                json!({"cmd": "set-size", "width": w, "height": h})
            } else if args.len() == 1 {
                let s = &args[0];
                if s.starts_with('+') || s.starts_with('-') {
                    let delta: i32 = s.parse().context("Invalid delta")?;
                    json!({"cmd": "resize-by", "delta": delta})
                } else {
                    anyhow::bail!("Size requires either W H or +/-N");
                }
            } else {
                anyhow::bail!("Size requires either W H or +/-N");
            }
        },
        Commands::Reload => json!({"cmd": "reload-config"}),
        Commands::State => json!({"cmd": "get-state"}),
        Commands::Quit => json!({"cmd": "quit"}),
    };

    let resp = send_command(&sock, cmd)?;

    if let Some(true) = resp.get("ok").and_then(|v| v.as_bool()) {
        // For state command, print the full response
        if matches!(&cli.command, Commands::State) {
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
    } else {
        let err = resp.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error");
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }

    Ok(())
}
