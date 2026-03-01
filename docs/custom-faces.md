# Custom Faces

Clockie supports SVG (and raster PNG/JPEG) images as analogue clock face backgrounds. Hands are always drawn procedurally on top -- the face image provides everything else (bezel, ticks, numerals, decorations).

## SVG requirements

- **ViewBox:** `200x200` recommended (`viewBox="0 0 200 200"`). The SVG is scaled to fit the clock diameter, so any square viewBox works, but 200x200 matches the bundled presets.
- **No hands:** Do not include hour/minute/second hands -- clockie draws these procedurally using the `[analogue]` and `[theme]` config sections.
- **Static only:** No embedded scripts, animations, or external references. Clockie renders SVGs with resvg, which supports static SVG elements only.
- **Center at (100, 100):** Place the clock center at the middle of the viewBox for correct hand alignment.

## Bundled presets

Clockie ships with 4 SVG face presets, installed to `$PREFIX/share/clockie/faces/`:

| Name | Description |
|------|-------------|
| `classic` | Traditional: bezel ring, hour/minute ticks, Arabic numerals at 12/3/6/9 |
| `minimal` | Clean: thin hour markers, no numerals, subtle center dot |
| `modern` | Bold: thick bar indices, thin minute lines, accent ring |
| `bare` | Filled circle with subtle edge -- blank canvas for procedural hands |

Use them with:

```toml
[background]
face_preset = "classic"
```

## Preset resolution

When `face_preset` is set to a name (not a path), clockie searches for `{name}.svg` in these directories, in order:

1. `$XDG_DATA_HOME/clockie/faces/` (default: `~/.local/share/clockie/faces/`)
2. Each directory in `$XDG_DATA_DIRS` + `/clockie/faces/` (default: `/usr/local/share/clockie/faces/`, `/usr/share/clockie/faces/`)

The first match wins. This means user-installed faces in `~/.local/share/clockie/faces/` take priority over system-installed ones.

If `face_preset` contains a path separator (`/`) or ends with `.svg`/`.svgz`, it is treated as a direct file path (with `~` expansion).

## Installing custom faces

Copy your SVG file to the user faces directory:

```sh
mkdir -p ~/.local/share/clockie/faces
cp my-clock-face.svg ~/.local/share/clockie/faces/
```

Then reference it by name (without the `.svg` extension):

```toml
[background]
face_preset = "my-clock-face"
```

Or use a direct path:

```toml
[background]
face_preset = "~/my-faces/custom.svg"
```

## Combining SVG faces with procedural elements

When an SVG face is loaded, clockie still draws hands procedurally. You can customise them in `[analogue]`:

```toml
[background]
face_preset = "minimal"

[analogue]
# Use arrow-tipped hands with taper
hand_cap = "arrow"
hand_taper = 0.4
hand_shadow = true

# Disable procedural ticks/numerals (the SVG provides these)
show_ticks = "none"
numerals = "none"
```

### What affects SVG faces vs procedural-only rendering

| Config area | SVG face loaded | No SVG (procedural) |
|-------------|-----------------|---------------------|
| `[analogue]` hands (`hand_cap`, lengths, widths, `hand_shadow`, `hand_taper`) | Drawn on top of SVG | Drawn on top of procedural face |
| `[analogue]` ticks (`show_ticks`, `tick_style`) | Drawn on top of SVG (usually set to `"none"`) | Drawn as part of procedural face |
| `[analogue]` numerals | Drawn on top of SVG (usually set to `"none"`) | Drawn as part of procedural face |
| `[analogue]` decorations (`face_fill`, `bezel_*`, `minute_track_*`) | Drawn under SVG (usually not visible) | Drawn as part of procedural face |
| `[theme]` hand colours | Always applies | Always applies |
| `[theme]` `tick_color` | Applies if ticks enabled | Applies |
