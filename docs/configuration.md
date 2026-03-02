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
| `text_outline` | boolean | `true` | Draw a contrasting outline around all text for readability |
| `auto_contrast` | string | `"auto"` | Auto-contrast mode: `"auto"`, `"always"`, or `"never"` |

**Auto-contrast** automatically picks a light or dark text colour based on the background brightness. This is especially useful when gallery images cycle through backgrounds of varying brightness.

- `"auto"` -- activates only when a gallery is configured (digital or analogue)
- `"always"` -- always samples the background and adapts text colour, even with a single static image
- `"never"` -- always uses the configured `fg_color`

When auto-contrast determines the background is light (luminance > 140), it switches to dark text (`#1a1a1a`). Otherwise it uses the configured `fg_color`.

**Text outline** draws all text at 8 compass offsets in a contrasting colour (dark outline for light text, light for dark), then the actual text on top. The outline radius scales with font size. This ensures text remains readable regardless of the background. Set `text_outline = false` to disable.

## [background]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `digital_image` | string | `""` | Path to PNG/JPEG background for digital face (empty = solid `bg_color`) |
| `analogue_face_image` | string | `""` | Path to PNG/JPEG for the analogue clock face (replaces procedural tick marks) |
| `face_preset` | string | `""` | Bundled preset name or path to an SVG face file (see below) |
| `image_scale` | string | `"fill"` | Scale mode: `"fill"`, `"fit"`, `"stretch"`, or `"center"` |
| `digital_gallery` | string or array | unset | Gallery for digital mode: a folder path (all images inside) or an explicit list of paths |
| `analogue_gallery` | string or array | unset | Gallery for analogue mode: a folder path (all images inside) or an explicit list of paths |
| `gallery_interval` | integer | `0` | Auto-rotate interval in seconds. `0` = disabled. |

Paths support `~` for the home directory (e.g. `"~/Pictures/clock.png"`).

**Face presets:** Clockie ships with 4 bundled SVG clock faces. Set `face_preset` to one of the preset names to use it:

| Preset | Description |
|--------|-------------|
| `"classic"` | Traditional: bezel ring, hour/minute ticks, Arabic numerals at 12/3/6/9 |
| `"minimal"` | Clean: thin hour markers only, no numerals, subtle center dot |
| `"modern"` | Bold: thick bar indices at hours, thin minute lines, accent ring |
| `"bare"` | Just a filled circle with a subtle edge -- blank canvas for procedural hands |

You can also set `face_preset` to a direct path (e.g. `"~/my-faces/custom.svg"`). Presets are resolved via XDG data directories -- see [Custom Faces](custom-faces.md) for details.

**Priority order** for analogue face images: `analogue_gallery` > `face_preset` > `analogue_face_image`.

**Gallery:** Set `digital_gallery` or `analogue_gallery` to enable cycling for that mode. Use a folder path to include all images in that directory, or an explicit array to control the exact order. Use `clockie ctl gallery next`/`prev` to cycle manually, or set `gallery_interval` to auto-rotate. When unset, the single-image fields (`digital_image`/`analogue_face_image`) are used.

```toml
# Folder — all images inside are used, sorted by filename
analogue_gallery = "~/.config/clockie/faces/analogue/"

# Explicit list — full control over order
digital_gallery = ["~/wallpapers/a.png", "~/wallpapers/b.jpg"]
```

**Cycling bundled presets:** Set `analogue_gallery` to the special value `"bundled"` to cycle through the bundled SVG face presets. Clockie will automatically locate the installed presets directory (via XDG data dirs or relative to the executable), so this works on NixOS without any manual file copying.

```toml
analogue_gallery = "bundled"
gallery_interval = 300
```

**Scale modes:**
- `fill` -- scale to cover the entire area, cropping overflow (default)
- `fit` -- scale to fit within the area, letterboxing as needed
- `stretch` -- stretch to fill exactly, ignoring aspect ratio
- `center` -- place at original size, centred

## [analogue]

Controls the procedural elements of the analogue clock face: hands, tick marks, numerals, and decorations. These settings apply regardless of whether an SVG face image is used -- procedural hands are always drawn on top.

### Hands

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `hand_cap` | string | `"round"` | Hand tip style: `"round"`, `"flat"`, or `"arrow"` |
| `hand_taper` | float | `0.0` | Taper ratio from base to tip. `0.0` = uniform width, `1.0` = full taper (tip approaches zero width) |
| `hour_hand_length` | float | `0.55` | Hour hand length as fraction of radius |
| `hour_hand_width` | float | `0.06` | Hour hand width as fraction of radius |
| `minute_hand_length` | float | `0.75` | Minute hand length as fraction of radius |
| `minute_hand_width` | float | `0.04` | Minute hand width as fraction of radius |
| `second_hand_length` | float | `0.85` | Second hand length as fraction of radius |
| `second_hand_width` | float | `0.02` | Second hand width as fraction of radius |
| `hand_shadow` | boolean | `false` | Draw a subtle drop shadow behind each hand |

### Tick marks

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `show_ticks` | string | `"all60"` | Which ticks to show: `"all60"`, `"hours_only"`, `"quarters_only"`, or `"none"` |
| `tick_style` | string | `"line"` | Tick shape: `"line"`, `"dot"`, or `"diamond"` |

### Numerals

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `numerals` | string | `"none"` | Numeral labels: `"none"`, `"arabic"`, or `"roman"` |
| `numeral_size` | float | `0.18` | Numeral size as fraction of radius |
| `numeral_inset` | float | `0.15` | Distance from edge to numeral center, as fraction of radius |

### Decorations

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `face_fill` | hex string | *(none)* | Fill colour behind the procedural face (empty = transparent) |
| `bezel_width` | float | `0.0` | Bezel ring width as fraction of radius (`0` = thin 2px default) |
| `bezel_color` | hex string | `"FFFFFFFF"` | Bezel ring colour |
| `minute_track_width` | float | `0.0` | Minute track ring width as fraction of radius (`0` = hidden) |
| `minute_track_color` | hex string | `"CCCCCCFF"` | Minute track ring colour |

When an SVG face is loaded (via `face_preset` or gallery), procedural decorations like ticks, numerals, bezel, and face fill are typically redundant -- the SVG provides the visual elements. Set `show_ticks = "none"` and `numerals = "none"` to avoid drawing over the SVG. Hands are always drawn procedurally.

**Example -- arrow hands with roman numerals:**

```toml
[analogue]
hand_cap = "arrow"
hand_taper = 0.3
hand_shadow = true
numerals = "roman"
numeral_size = 0.16
show_ticks = "hours_only"
tick_style = "dot"
```

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

Sub-clocks respect the `hour_format` and `show_seconds` settings from `[clock]`. In compact mode, sub-clocks are hidden entirely. In analogue full mode, sub-clocks stack vertically (one per row, centred); in digital mode they are arranged side by side.

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
text_outline      = true
auto_contrast     = "auto"

[background]
digital_image       = ""
analogue_face_image = ""
# face_preset       = "classic"
image_scale         = "fill"
# analogue_gallery = "~/.config/clockie/faces/analogue/"
# digital_gallery = ["~/wallpapers/a.png", "~/wallpapers/b.jpg"]
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
