# Compositor integration

Clockie uses the `wlr-layer-shell` protocol and works on any supporting Wayland compositor. This guide shows how to autostart clockie, bind hotkeys to `clockiectl`, and apply compositor-level rules.

## Hyprland

### Autostart

In `~/.config/hypr/hyprland.conf`:

```ini
exec-once = clockie
```

### Keybindings

```ini
# Toggle compact mode
bind = ALT, DOWN, exec, clockiectl compact toggle
# Toggle face (digital / analogue)
bind = ALT, SPACE, exec, clockiectl face toggle
# Toggle drag lock
bind = ALT, RIGHT, exec, clockiectl lock toggle
# Scale up / down
bind = ALT, EQUAL, exec, clockiectl size +10
bind = ALT, MINUS, exec, clockiectl size -10
# Gallery: next / previous image
bind = ALT, BRACKETRIGHT, exec, clockiectl gallery next
bind = ALT, BRACKETLEFT, exec, clockiectl gallery prev
```

### Layer rules

Hyprland layer rules match by the layer-shell namespace. Clockie's namespace is `clockie`.

```ini
# Disable blur behind the clock
layerrule = noblur, clockie
# Disable animations on the clock surface
layerrule = noanim, clockie
```

See the [Hyprland wiki on layer rules](https://wiki.hyprland.org/Configuring/Window-Rules/#layer-rules) for the full list of properties.

## niri

### Autostart

In `~/.config/niri/config.kdl`:

```kdl
spawn-at-startup "clockie"
```

### Keybindings

Add these inside `binds { }`:

```kdl
binds {
    // Toggle compact mode
    Alt+Down { spawn "clockiectl" "compact" "toggle"; }
    // Toggle face (digital / analogue)
    Alt+Space { spawn "clockiectl" "face" "toggle"; }
    // Toggle drag lock
    Alt+Right { spawn "clockiectl" "lock" "toggle"; }
    // Scale up / down
    Alt+Equal { spawn "clockiectl" "size" "+10"; }
    Alt+Minus { spawn "clockiectl" "size" "--" "-10"; }
    // Gallery: next / previous image
    Alt+BracketRight { spawn "clockiectl" "gallery" "next"; }
    Alt+BracketLeft { spawn "clockiectl" "gallery" "prev"; }
}
```

### Layer rules

niri matches layer-shell surfaces by namespace. Clockie's namespace is `clockie`.

```kdl
layer-rule {
    match namespace="clockie"
    // Example: prevent the clock from being captured in screencasts
    block-out-from "screencast"
}
```

See the [niri wiki on layer rules](https://github.com/YaLTeR/niri/wiki/Configuration:-Layer%E2%80%90Rules) for the full list of properties.

## Mango

### Autostart

In `~/.config/mango/config.conf` (or a sourced file):

```ini
exec-once=clockie
```

### Keybindings

```ini
# Toggle compact mode
bind=ALT,Down,spawn,clockiectl compact toggle
# Toggle face (digital / analogue)
bind=ALT,Space,spawn,clockiectl face toggle
# Toggle drag lock
bind=ALT,Right,spawn,clockiectl lock toggle
# Scale up / down
bind=ALT,Equal,spawn,clockiectl size +10
bind=ALT,Minus,spawn,clockiectl size -10
# Gallery: next / previous image
bind=ALT,BracketRight,spawn,clockiectl gallery next
bind=ALT,BracketLeft,spawn,clockiectl gallery prev
```

### Layer rules

Mango layer rules match by `layer_name`, which corresponds to the layer-shell namespace. Clockie's namespace is `clockie`.

```ini
# Disable blur behind the clock
layerrule=noblur:1,layer_name:clockie
# Disable shadow
layerrule=noshadow:1,layer_name:clockie
```

## Sway

### Autostart

In `~/.config/sway/config`:

```
exec clockie
```

### Keybindings

```
# Toggle compact mode
bindsym Alt+Down exec clockiectl compact toggle
# Toggle face (digital / analogue)
bindsym Alt+space exec clockiectl face toggle
# Toggle drag lock
bindsym Alt+Right exec clockiectl lock toggle
# Scale up / down
bindsym Alt+equal exec clockiectl size +10
bindsym Alt+minus exec clockiectl size -10
# Gallery: next / previous image
bindsym Alt+bracketright exec clockiectl gallery next
bindsym Alt+bracketleft exec clockiectl gallery prev
```

## General notes

- **Namespace:** Clockie registers its layer-shell surface with the namespace `clockie`. Use this when writing compositor rules.
- **Layer:** The default layer is `top` (above windows, below overlays). Change it with `layer = "overlay"` in `config.toml` if you want the clock to stay above fullscreen windows.
- **Multiple instances:** When using `--socket` to run multiple clockie instances, each still uses the `clockie` namespace — compositor rules apply to all instances.
