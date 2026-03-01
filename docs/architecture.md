# Architecture

## Module structure

```
src/
  main.rs                 CLI entry point, arg parsing, config loading
  config.rs               Configuration structs, TOML parsing, defaults
  ipc.rs                  IPC command/response types, socket handling
  battery.rs              Battery info from /sys/class/power_supply
  time_utils.rs           Time formatting, timezone conversion
  canvas.rs               Drawing primitives (Canvas, FontState), outlined text, luminance sampling, image loading
  wayland.rs              Wayland integration, event loop, IPC polling
  renderer/
    mod.rs                Size computation, bg/fg render dispatch, ContrastInfo, SubclockSizing
    digital.rs            Digital face rendering
    analogue.rs           Analogue face rendering
    subclock.rs           Timezone sub-clock rendering
    battery.rs            Battery indicator rendering
  ctl.rs                  Control client (clockie ctl subcommand)
```

## Rendering pipeline

Rendering is split into background and foreground phases with a contrast-sampling step in between:

1. **Size computation** (`renderer::compute_size`) -- measures text and computes the required window dimensions based on `font_size`/`diameter`, compact state, date visibility, battery, and timezone count
2. **Canvas creation** -- a `tiny-skia` pixmap is created at the computed dimensions
3. **Background phase** (`renderer::render_background`) -- solid colour fill or scaled background image with colour scrim (digital), or clear + face image/procedural ticks (analogue)
4. **Contrast resolution** -- if auto-contrast is active and the background changed (gallery rotate/next/prev), the canvas is sampled for average perceptual luminance. Light backgrounds (luminance > 140) trigger dark text; otherwise the configured `fg_color` is used. The result is cached until the next background change.
5. **Foreground phase** (`renderer::render_foreground`) -- digital text or analogue hands/boss, battery overlay, and timezone sub-clocks. All text uses the resolved contrast colour and optional outline rendering.
6. **Opacity** -- per-pixel alpha scaling if opacity < 1.0
7. **Pixel format conversion** -- RGBA to BGRA (ARGB8888 little-endian) for Wayland
8. **Buffer commit** -- attached to the Wayland surface and committed

### Text rendering

Text can be drawn in two modes depending on the `text_outline` config:
- **Plain** -- standard alpha-blended text
- **Outlined** -- text is drawn 9 times: once at each of 8 compass offsets in a contrasting colour (dark for light text, light for dark), then the actual text on top. The outline radius scales with font size: `(size * 0.04).max(0.8).min(1.5)` pixels.

## Event loop

The main loop runs at approximately 10 Hz (100ms poll timeout) and redraws at 1 Hz (when the system second changes). IPC commands are processed on each loop iteration via non-blocking socket accept.

## Font loading

Fonts are resolved in this order:
1. Direct file path (if `font` is a path to a `.ttf`/`.otf` file)
2. System font directories (`/usr/share/fonts`, `/usr/local/share/fonts`, Nix profile paths)
3. Hardcoded fallbacks (DejaVu Sans Mono, Liberation Mono)
4. Nix store search (`/nix/store/*dejavu-fonts*`, `*liberation-fonts*`)

If no font is found, the process panics with a message asking the user to install a TTF font.
