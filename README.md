# clockie

A lightweight Wayland layer-shell desktop clock widget written in Rust. Renders directly on the desktop layer using the `wlr-layer-shell` protocol, sitting above the wallpaper and below windows.

## Features

- **Digital and analogue clock faces** with runtime switching
- **Content-driven auto-sizing** -- the window sizes itself to fit the content
- **Up to 2 timezone sub-clocks** displayed beneath the primary clock
- **Battery indicator** with colour-coded charge level and charging animation
- **Custom background images** for both digital and analogue faces (PNG/JPEG)
- **Full theming** -- foreground, background, hand colours, tick marks, all in hex RGBA
- **Drag-to-move** with cross-monitor support
- **IPC control** via Unix socket (`clockiectl`)
- **TOML configuration** with sensible defaults generated on first run

## Requirements

- A Wayland compositor with `wlr-layer-shell-unstable-v1` support (e.g. Sway, Hyprland, niri, river)
- A TrueType or OpenType font installed on the system

## Installation

### Nix flake

```nix
# flake.nix input
clockie.url = "github:barrulus/clockie";

# Add to packages
clockie.packages.${system}.default
```

Shell completions for bash, zsh, and fish are installed automatically.

### From source

```sh
make install              # installs to /usr/local
make install PREFIX=~/.local  # or a custom prefix
```

### Cargo

```sh
cargo install --path .
```

With `cargo install`, generate completions manually:

```sh
clockie --completions bash > ~/.local/share/bash-completion/completions/clockie
clockiectl completions bash > ~/.local/share/bash-completion/completions/clockiectl
```

## Quick start

```sh
# Launch with defaults (digital face, top-right corner)
clockie

# Analogue face in compact mode
clockie --face analogue --compact

# With timezone sub-clocks
clockie --tz1 Europe/London --tz2 America/New_York
```

On first run, a default config file is generated at `~/.config/clockie/config.toml`.

## Documentation

- [Configuration](docs/configuration.md) -- all config fields and examples
- [CLI usage](docs/cli.md) -- `clockie` and `clockiectl` commands
- [IPC protocol](docs/ipc.md) -- JSON socket protocol for programmatic control
- [Multi-monitor](docs/multi-monitor.md) -- cross-monitor drag and output switching
- [Architecture](docs/architecture.md) -- module structure and rendering pipeline

## Tips

- **Quick resize:** Bind `clockiectl size +10` / `clockiectl size -10` to compositor hotkeys.
- **Face switching:** Bind `clockiectl face toggle` for one-key switching.
- **Multiple instances:** Use `--socket` to run separate clockie instances.
- **Transparent background:** Set `bg_color` alpha to `00` (e.g. `"1a1a2e00"`) for a fully transparent background.

## Logging

```sh
RUST_LOG=info clockie     # default
RUST_LOG=debug clockie    # verbose
```

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).
