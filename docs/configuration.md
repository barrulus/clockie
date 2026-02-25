# Configuration

**Location:** `~/.config/clockie/config.toml` (respects `$XDG_CONFIG_HOME`)

The config is TOML-formatted with the following sections. All fields are optional; defaults are shown. A default config is generated on first run.

## [window]

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
| `output` | string | *(none)* | Output/monitor to display on (e.g. `"HDMI-A-1"`). Omit for compositor default. |

**Anchor examples:**
- `"top right"` -- top-right corner (default)
- `"bottom left"` -- bottom-left corner
- `"top"` -- centred along top edge
- `"top bottom right"` -- stretched along right edge

## [clock]

Controls the clock display and content sizing.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `face` | string | `"digital"` | Clock face mode: `"digital"` or `"analogue"` |
| `hour_format` | integer | `12` | `12` for 12-hour (with AM/PM) or `24` for 24-hour |
| `show_seconds` | boolean | `true` | Show seconds in time display |
| `show_date` | boolean | `true` | Show date line below time (digital face, non-compact only) |
| `date_format` | string | `"%A, %d %B %Y"` | Date format using [chrono strftime](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) syntax |
| `font` | string | `"monospace"` | Font name or path to a `.ttf`/`.otf` file |
| `font_size` | float | `48.0` | Main time text size in pixels (digital mode). Minimum: 10.0 |
| `diameter` | integer | `180` | Clock face diameter in pixels (analogue mode). Minimum: 40 |

**Content-driven sizing:** The `font_size` (digital) and `diameter` (analogue) settings control how large the content is drawn. The window automatically sizes itself to wrap the content with appropriate padding.

## [theme]

All colours are specified in `RRGGBB` or `RRGGBBAA` hex format. The `#` prefix is optional.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `fg_color` | hex string | `"FFFFFFFF"` | Foreground/text colour |
| `bg_color` | hex string | `"1a1a2eCC"` | Background colour |
| `hour_hand_color` | hex string | `"FFFFFFFF"` | Analogue hour hand colour |
| `minute_hand_color` | hex string | `"FFFFFFFF"` | Analogue minute hand colour |
| `second_hand_color` | hex string | `"ef4444FF"` | Analogue second hand colour |
| `tick_color` | hex string | `"CCCCCCFF"` | Tick mark colour on procedural analogue face |

## [background]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `digital_image` | string | `""` | Path to PNG/JPEG background for digital face (empty = solid `bg_color`) |
| `analogue_face_image` | string | `""` | Path to PNG/JPEG for the analogue clock face (replaces procedural tick marks) |
| `image_scale` | string | `"fill"` | Scale mode: `"fill"`, `"fit"`, `"stretch"`, or `"center"` |
| `digital_images` | array of strings | `[]` | Gallery of digital background images (overrides `digital_image` when non-empty) |
| `analogue_face_images` | array of strings | `[]` | Gallery of analogue face images (overrides `analogue_face_image` when non-empty) |
| `gallery_interval` | integer | `0` | Auto-rotate interval in seconds. `0` = disabled. |

Paths support `~` for the home directory (e.g. `"~/Pictures/clock.png"`).

**Gallery:** When `digital_images` or `analogue_face_images` is non-empty, the gallery system is active for that mode. Use `clockiectl gallery next`/`prev` to cycle manually, or set `gallery_interval` to auto-rotate. If the gallery arrays are empty, the single-image fields (`digital_image`/`analogue_face_image`) are used as before.

**Scale modes:**
- `fill` -- scale to cover the entire area, cropping overflow (default)
- `fit` -- scale to fit within the area, letterboxing as needed
- `stretch` -- stretch to fill exactly, ignoring aspect ratio
- `center` -- place at original size, centred

## [battery]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `false` | Show battery indicator in the top-right corner |
| `show_percentage` | boolean | `true` | Display percentage text next to the battery icon |

Battery data is read from `/sys/class/power_supply/BAT*`. The icon colour changes based on charge level (green >50%, yellow 21--50%, red <=20%). A lightning bolt is drawn when charging.

## [[timezone]]

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

## Example config

```toml
[window]
layer   = "top"
anchor  = "top right"
margin_top  = 20
margin_right = 20
opacity = 0.9

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
# digital_images = ["~/wallpapers/a.png", "~/wallpapers/b.jpg"]
# analogue_face_images = ["~/faces/classic.png", "~/faces/minimal.png"]
# gallery_interval = 300

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
