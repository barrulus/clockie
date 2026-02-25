# Architecture

## Module structure

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

## Rendering pipeline

1. **Size computation** (`renderer::compute_size`) -- measures text and computes the required window dimensions based on `font_size`/`diameter`, compact state, date visibility, battery, and timezone count
2. **Canvas creation** -- a `tiny-skia` pixmap is created at the computed dimensions
3. **Background** -- solid colour fill or scaled background image with colour scrim
4. **Face rendering** -- digital text or analogue hands/ticks
5. **Battery overlay** -- icon and percentage text in the top-right corner
6. **Sub-clocks** -- timezone labels and times in a footer area
7. **Opacity** -- per-pixel alpha scaling if opacity < 1.0
8. **Pixel format conversion** -- RGBA to BGRA (ARGB8888 little-endian) for Wayland
9. **Buffer commit** -- attached to the Wayland surface and committed

## Event loop

The main loop runs at approximately 10 Hz (100ms poll timeout) and redraws at 1 Hz (when the system second changes). IPC commands are processed on each loop iteration via non-blocking socket accept.

## Font loading

Fonts are resolved in this order:
1. Direct file path (if `font` is a path to a `.ttf`/`.otf` file)
2. System font directories (`/usr/share/fonts`, `/usr/local/share/fonts`, Nix profile paths)
3. Hardcoded fallbacks (DejaVu Sans Mono, Liberation Mono)
4. Nix store search (`/nix/store/*dejavu-fonts*`, `*liberation-fonts*`)

If no font is found, the process panics with a message asking the user to install a TTF font.
