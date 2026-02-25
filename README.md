# clockie

A lightweight Wayland layer-shell desktop clock widget written in Rust. Renders directly on the desktop layer using the `wlr-layer-shell` protocol, sitting above the wallpaper and below windows.

## Features

- **Digital and analogue clock faces** with runtime switching
- **Content-driven auto-sizing** -- the window sizes itself to fit the content; changing font size, diameter, face mode, or toggling compact mode automatically resizes the window
- **Up to 2 timezone sub-clocks** displayed beneath the primary clock
- **Battery indicator** with colour-coded charge level and charging animation
- **Custom background images** for both digital and analogue faces (PNG/JPEG)
- **Full theming** -- foreground, background, hand colours, tick marks, all in hex RGBA
- **Drag-to-move** -- click and drag the clock to reposition it; margins are persisted to config on release
- **IPC control** via Unix socket -- switch faces, toggle compact mode, resize, reload config, query state
- **Compact mode** for a smaller footprint
- **TOML configuration** with sensible defaults generated on first run
- **Minimal resource footprint** -- no Electron, no GTK/Qt, pure Rust

## Requirements

- A Wayland compositor with `wlr-layer-shell-unstable-v1` support (e.g. Sway, Hyprland, niri, river)
- A TrueType or OpenType font installed on the system (DejaVu Sans Mono, Liberation Mono, or any `.ttf`/`.otf`)

## Building

```sh
cargo build --release
```

This produces two binaries:
- `target/release/clockie` -- the clock daemon
- `target/release/clockiectl` -- the control client

### Nix

A `flake.nix` is included for Nix users:

```sh
nix develop   # enter dev shell with all dependencies
nix build     # build the package
```

## Installation

Copy the two binaries somewhere on your `$PATH`:

```sh
install -Dm755 target/release/clockie   ~/.local/bin/clockie
install -Dm755 target/release/clockiectl ~/.local/bin/clockiectl
```

## Quick start

```sh
# Launch with defaults (digital face, top-right corner, 48px font)
clockie

# Launch with analogue face in compact mode
clockie --face analogue --compact

# Launch with two timezone sub-clocks
clockie --tz1 Europe/London --tz2 America/New_York
```

On first run, a default config file is generated at `~/.config/clockie/config.toml`.

---

## Configuration

**Location:** `~/.config/clockie/config.toml` (respects `$XDG_CONFIG_HOME`)

The config is TOML-formatted with the following sections. All fields are optional; defaults are shown.

### [window]

Controls window placement and appearance. The window size is computed automatically from the content -- there are no width/height settings.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `layer` | string | `"top"` | Wayland layer: `"background"`, `"bottom"`, `"top"`, or `"overlay"` |
| `anchor` | string | `"top right"` | Anchor edges, space-separated: `top`, `bottom`, `left`, `right` |
| `margin_top` | integer | `20` | Margin from top edge in pixels |
| `margin_bottom` | integer | `0` | Margin from bottom edge |
| `margin_left` | integer | `0` | Margin from left edge |
| `margin_right` | integer | `20` | Margin from right edge |
| `opacity` | float | `1.0` | Window opacity, 0.0 (invisible) to 1.0 (opaque) |
| `compact` | boolean | `false` | Start in compact mode |

**Anchor examples:**
- `"top right"` -- top-right corner (default)
- `"bottom left"` -- bottom-left corner
- `"top"` -- centred along top edge
- `"top bottom right"` -- stretched along right edge

### [clock]

Controls the clock display and content sizing.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `face` | string | `"digital"` | Clock face mode: `"digital"` or `"analogue"` |
| `hour_format` | integer | `12` | `12` for 12-hour (with AM/PM) or `24` for 24-hour |
| `show_seconds` | boolean | `true` | Show seconds in time display |
| `show_date` | boolean | `true` | Show date line below time (digital face, non-compact only) |
| `date_format` | string | `"%A, %d %B %Y"` | Date format using [chrono strftime](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) syntax |
| `font` | string | `"monospace"` | Font name or path to a `.ttf`/`.otf` file |
| `font_size` | float | `48.0` | Main time text size in pixels (digital mode). The window auto-sizes to fit. Minimum: 10.0 |
| `diameter` | integer | `180` | Clock face diameter in pixels (analogue mode). The window auto-sizes to fit. Minimum: 40 |

**Content-driven sizing:** The `font_size` (digital) and `diameter` (analogue) settings control how large the content is drawn. The window automatically sizes itself to wrap the content with appropriate padding. Changing these values, switching face mode, toggling compact, or adding/removing timezones all trigger an automatic resize.

### [theme]

All colours are specified in `RRGGBB` or `RRGGBBAA` hex format. The `#` prefix is optional.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `fg_color` | hex string | `"FFFFFFFF"` | Foreground/text colour |
| `bg_color` | hex string | `"1a1a2eCC"` | Background colour (also used as scrim over background images) |
| `hour_hand_color` | hex string | `"FFFFFFFF"` | Analogue hour hand colour |
| `minute_hand_color` | hex string | `"FFFFFFFF"` | Analogue minute hand colour |
| `second_hand_color` | hex string | `"ef4444FF"` | Analogue second hand colour |
| `tick_color` | hex string | `"CCCCCCFF"` | Tick mark colour on procedural analogue face |

### [background]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `digital_image` | string | `""` | Path to PNG/JPEG background for digital face (empty = solid `bg_color`) |
| `analogue_face_image` | string | `""` | Path to PNG/JPEG for the analogue clock face (replaces procedural tick marks) |
| `image_scale` | string | `"fill"` | Scale mode: `"fill"`, `"fit"`, `"stretch"`, or `"center"` |

**Scale modes:**
- `fill` -- scale to cover the entire area, cropping overflow (default)
- `fit` -- scale to fit within the area, letterboxing as needed
- `stretch` -- stretch to fill exactly, ignoring aspect ratio
- `center` -- place at original size, centred

### [battery]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Show battery indicator in the top-right corner |
| `show_percentage` | boolean | `true` | Display percentage text next to the battery icon |

Battery data is read from `/sys/class/power_supply/BAT*`. The icon colour changes based on charge level:
- Green: >50%
- Yellow: 21--50%
- Red: <=20%

A lightning bolt is drawn over the icon when the battery is charging.

### [[timezone]]

Up to 2 timezone sub-clocks can be configured. Each is a separate `[[timezone]]` entry.

```toml
[[timezone]]
label = "London"
tz    = "Europe/London"

[[timezone]]
label = "New York"
tz    = "America/New_York"
```

| Field | Type | Description |
|-------|------|-------------|
| `label` | string | Display label shown above the timezone time |
| `tz` | string | [IANA timezone](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones) identifier |

Sub-clocks respect the `hour_format` and `show_seconds` settings from `[clock]`.

### Example config

```toml
[window]
layer   = "top"
anchor  = "top right"
margin_top  = 20
margin_right = 20
opacity = 0.9
compact = false

[clock]
face         = "digital"
hour_format  = 24
show_seconds = true
show_date    = true
date_format  = "%A, %d %B %Y"
font         = "monospace"
font_size    = 56.0
diameter     = 200

[theme]
fg_color          = "FFFFFFFF"
bg_color          = "1a1a2eCC"
second_hand_color = "ef4444FF"
tick_color        = "CCCCCCFF"

[background]
digital_image       = ""
analogue_face_image = ""
image_scale         = "fill"

[battery]
enabled         = true
show_percentage = true

[[timezone]]
label = "London"
tz    = "Europe/London"

[[timezone]]
label = "Tokyo"
tz    = "Asia/Tokyo"
```

---

## Shell completions

Both `clockie` and `clockiectl` can generate shell completion scripts for bash, zsh, fish, and elvish.

```sh
# Bash
clockiectl completions bash > ~/.local/share/bash-completion/completions/clockiectl
clockie --completions bash > ~/.local/share/bash-completion/completions/clockie

# Zsh (add the directory to $fpath before compinit)
clockiectl completions zsh > ~/.zfunc/_clockiectl
clockie --completions zsh > ~/.zfunc/_clockie

# Fish
clockiectl completions fish > ~/.config/fish/completions/clockiectl.fish
clockie --completions fish > ~/.config/fish/completions/clockie.fish
```

---

## CLI usage

### clockie (daemon)

```
clockie [OPTIONS]

Options:
  -c, --config <PATH>    Path to config file [default: ~/.config/clockie/config.toml]
      --face <MODE>      Override face mode: digital or analogue
      --compact          Start in compact mode
      --tz1 <TZ>         Override first timezone (e.g. Europe/London)
      --tz2 <TZ>         Override second timezone (e.g. America/New_York)
      --no-tz            Disable timezone sub-clocks
      --socket <PATH>    Override IPC socket path
      --completions <SHELL>  Generate shell completions (bash, zsh, fish, elvish)
  -h, --help             Print help
  -V, --version          Print version
```

**Examples:**

```sh
# Default digital clock
clockie

# Analogue face, compact, no sub-clocks
clockie --face analogue --compact --no-tz

# Custom config location
clockie -c ~/my-clockie.toml

# Override timezones from CLI
clockie --tz1 Europe/London --tz2 America/New_York
```

### clockiectl (control client)

```
clockiectl [--socket <PATH>] <COMMAND>

Commands:
  face <MODE>       Set or toggle clock face (digital, analogue, toggle)
  compact <MODE>    Control compact mode (on, off, toggle)
  lock <MODE>       Control drag lock (on, off, toggle)
  size <ARGS>       Set content size or scale by delta
  reload            Reload configuration file
  state             Print current state as JSON
  quit              Shut down clockie
  completions <SHELL>  Generate shell completions (bash, zsh, fish, elvish)
```

#### face

```sh
clockiectl face digital    # switch to digital
clockiectl face analogue   # switch to analogue
clockiectl face toggle     # toggle between them
```

Switching face mode automatically resizes the window to fit the new content.

#### compact

```sh
clockiectl compact on      # enable compact mode
clockiectl compact off     # disable compact mode
clockiectl compact toggle  # toggle
```

Compact mode reduces the time text to 70% of `font_size` (digital) or the face to 75% of `diameter` (analogue), and hides the date line.

#### lock

```sh
clockiectl lock on      # prevent dragging
clockiectl lock off     # allow dragging
clockiectl lock toggle  # toggle drag lock
```

When locked, pointer drags are ignored and the clock stays in place.

#### size

The `size` command adjusts `font_size` (digital mode) or `diameter` (analogue mode). The window auto-resizes after any change.

```sh
# Set font size directly (for digital mode)
clockiectl size 64

# Scale up by 10 (adds to font_size or diameter, depending on current face)
clockiectl size +10

# Scale down by 5
clockiectl size -5

# Explicitly set font size
clockiectl size font 72

# Explicitly set analogue diameter
clockiectl size diameter 240
```

Minimum values: font size 10.0, diameter 40.

#### reload

```sh
clockiectl reload
```

Re-reads the config file from disk. Preserves the current face mode and compact state (so IPC-toggled values aren't lost). Applies changes to: colours, font, margins, anchor, layer, background images, battery settings, timezones, font_size, diameter.

#### state

```sh
clockiectl state
```

Prints the current state as JSON:

```json
{
  "ok": true,
  "face": "digital",
  "compact": false,
  "width": 352,
  "height": 98,
  "font_size": 48.0,
  "diameter": 180,
  "config_path": "/home/user/.config/clockie/config.toml",
  "locked": false
}
```

#### quit

```sh
clockiectl quit
```

Shuts down the clockie daemon cleanly.

---

## IPC protocol

For programmatic control beyond `clockiectl`, you can send JSON commands directly to the Unix socket.

**Socket location:** `$XDG_RUNTIME_DIR/clockie.sock` (fallback: `/tmp/clockie-$UID.sock`)

**Protocol:** Send a single JSON object followed by a newline (`\n`). Read one JSON line back as the response.

### Commands

| Command | JSON |
|---------|------|
| Set face | `{"cmd": "set-face", "face": "digital"}` |
| Toggle face | `{"cmd": "toggle-face"}` |
| Set compact | `{"cmd": "set-compact", "compact": true}` |
| Toggle compact | `{"cmd": "toggle-compact"}` |
| Set font size | `{"cmd": "set-font-size", "size": 64.0}` |
| Set diameter | `{"cmd": "set-diameter", "diameter": 200}` |
| Scale by delta | `{"cmd": "scale-by", "delta": 10}` |
| Set locked | `{"cmd": "set-locked", "locked": true}` |
| Toggle locked | `{"cmd": "toggle-locked"}` |
| Reload config | `{"cmd": "reload-config"}` |
| Get state | `{"cmd": "get-state"}` |
| Quit | `{"cmd": "quit"}` |

### Responses

**Success:**
```json
{"ok": true}
```

**Error:**
```json
{"ok": false, "error": "Description of the error"}
```

**State (get-state only):**
```json
{
  "ok": true,
  "face": "digital",
  "compact": false,
  "width": 352,
  "height": 98,
  "font_size": 48.0,
  "diameter": 180,
  "config_path": "/home/user/.config/clockie/config.toml",
  "locked": false
}
```

### Example with socat

```sh
echo '{"cmd":"get-state"}' | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/clockie.sock
```

---

## Architecture

### Module structure

```
src/
  main.rs                 CLI entry point, arg parsing, config loading
  config.rs               Configuration structs, TOML parsing, defaults
  ipc.rs                  IPC command/response types, socket handling
  battery.rs              Battery info from /sys/class/power_supply
  time_utils.rs           Time formatting, timezone conversion
  canvas.rs               Drawing primitives (Canvas, FontState), image loading
  wayland.rs              Wayland integration, event loop, IPC polling
  renderer/
    mod.rs                Size computation, render dispatch
    digital.rs            Digital face rendering
    analogue.rs           Analogue face rendering
    subclock.rs           Timezone sub-clock rendering
    battery.rs            Battery indicator rendering
  bin/
    clockiectl.rs         Control client binary
```

### Rendering pipeline

1. **Size computation** (`renderer::compute_size`) -- measures text and computes the required window dimensions based on `font_size`/`diameter`, compact state, date visibility, battery, and timezone count
2. **Canvas creation** -- a `tiny-skia` pixmap is created at the computed dimensions
3. **Background** -- solid colour fill or scaled background image with colour scrim
4. **Face rendering** -- digital text or analogue hands/ticks
5. **Battery overlay** -- icon and percentage text in the top-right corner
6. **Sub-clocks** -- timezone labels and times in a footer area
7. **Opacity** -- per-pixel alpha scaling if opacity < 1.0
8. **Pixel format conversion** -- RGBA to BGRA (ARGB8888 little-endian) for Wayland
9. **Buffer commit** -- attached to the Wayland surface and committed

### Event loop

The main loop runs at approximately 10 Hz (100ms poll timeout) and redraws at 1 Hz (when the system second changes). IPC commands are processed on each loop iteration via non-blocking socket accept.

### Font loading

Fonts are resolved in this order:
1. Direct file path (if `font` is a path to a `.ttf`/`.otf` file)
2. System font directories (`/usr/share/fonts`, `/usr/local/share/fonts`, Nix profile paths)
3. Hardcoded fallbacks (DejaVu Sans Mono, Liberation Mono)
4. Nix store search (`/nix/store/*dejavu-fonts*`, `*liberation-fonts*`)

If no font is found, the process panics with a message asking the user to install a TTF font.

---

## Logging

clockie uses `env_logger`. Set the `RUST_LOG` environment variable to control log output:

```sh
RUST_LOG=info clockie        # default -- startup info
RUST_LOG=debug clockie       # verbose
RUST_LOG=warn clockie        # warnings and errors only
```

---

## Tips

- **Quick resize from the keyboard:** Bind `clockiectl size +10` and `clockiectl size -10` to hotkeys in your compositor for quick font size adjustment.
- **Face switching hotkey:** Bind `clockiectl face toggle` for one-key switching between digital and analogue.
- **Multiple instances:** Use `--socket` to run multiple clockie instances with separate IPC channels.
- **Transparent background:** Set `bg_color` alpha to `00` (e.g. `"1a1a2e00"`) and `opacity = 1.0` for a fully transparent background with visible text.
- **Date format:** The `date_format` field uses [chrono strftime syntax](https://docs.rs/chrono/latest/chrono/format/strftime/index.html). Common patterns:
  - `"%A, %d %B %Y"` -- Monday, 24 February 2026
  - `"%d/%m/%Y"` -- 24/02/2026
  - `"%Y-%m-%d"` -- 2026-02-24
  - `"%a %b %e"` -- Mon Feb 24

## License

See [LICENSE](LICENSE) for details.
