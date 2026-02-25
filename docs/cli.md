# CLI usage

## clockie (daemon)

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

## clockiectl (control client)

```
clockiectl [--socket <PATH>] <COMMAND>

Commands:
  face <MODE>       Set or toggle clock face (digital, analogue, toggle)
  compact <MODE>    Control compact mode (on, off, toggle)
  lock <MODE>       Control drag lock (on, off, toggle)
  size <ARGS>       Set content size or scale by delta
  output <NAME>     Move clock to a named output (or "next"/"prev" to cycle)
  gallery <ACTION>  Control background image gallery
  reload            Reload configuration file
  state             Print current state as JSON
  quit              Shut down clockie
  completions <SHELL>  Generate shell completions (bash, zsh, fish, elvish)
```

### face

```sh
clockiectl face digital    # switch to digital
clockiectl face analogue   # switch to analogue
clockiectl face toggle     # toggle between them
```

Switching face mode automatically resizes the window to fit the new content.

### compact

```sh
clockiectl compact on      # enable compact mode
clockiectl compact off     # disable compact mode
clockiectl compact toggle  # toggle
```

Compact mode reduces the time text to 70% of `font_size` (digital) or the face to 75% of `diameter` (analogue), and hides the date line.

### lock

```sh
clockiectl lock on      # prevent dragging
clockiectl lock off     # allow dragging
clockiectl lock toggle  # toggle drag lock
```

When locked, pointer drags are ignored and the clock stays in place.

### size

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

### output

```sh
clockiectl output HDMI-A-1   # move to a specific output
clockiectl output next       # cycle to the next output
clockiectl output prev       # cycle to the previous output
```

The output name is persisted to config. You can also drag the clock across monitor edges -- see [Multi-monitor](multi-monitor.md).

### gallery

Control the background image gallery. Operates on whichever face mode is currently active (digital or analogue).

```sh
clockiectl gallery next                # advance to next image
clockiectl gallery prev                # go back to previous image
clockiectl gallery set 2               # jump to image at index 2
clockiectl gallery start               # start auto-rotate (uses configured interval)
clockiectl gallery start --interval 5  # start auto-rotate with 5-second interval
clockiectl gallery stop                # stop auto-rotate
clockiectl gallery interval 10         # set rotate interval to 10 seconds
```

Requires `digital_images` or `analogue_face_images` to be configured in `[background]`. See [Configuration](configuration.md#background) for details.

### reload

```sh
clockiectl reload
```

Re-reads the config file from disk. Preserves the current face mode and compact state. Applies changes to: colours, font, margins, anchor, layer, background images, gallery arrays, battery settings, timezones, font_size, diameter.

### state

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
  "locked": false,
  "output": "eDP-1",
  "gallery_digital_index": 0,
  "gallery_analogue_index": 0,
  "gallery_digital_count": 3,
  "gallery_analogue_count": 2,
  "gallery_rotate_active": true,
  "gallery_rotate_interval": 300
}
```

### quit

```sh
clockiectl quit
```

Shuts down the clockie daemon cleanly.
