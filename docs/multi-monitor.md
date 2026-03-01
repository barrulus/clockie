# Multi-monitor support

Clockie can move between monitors in two ways: dragging to the screen edge, and explicit IPC commands.

## Drag to edge

When you drag the clock to the edge of a monitor that borders another monitor, the clock automatically moves to the adjacent output. The anchor flips to the arriving edge (e.g. dragging left off the screen re-anchors to the right side of the new output) and the margin on that edge resets to 0.

Both the output name and margins are persisted to config on move.

## IPC / clockie ctl

```sh
clockie ctl output HDMI-A-1   # move to a specific output by name
clockie ctl output next       # cycle to the next output
clockie ctl output prev       # cycle to the previous output
```

## Config persistence

The `output` field in `[window]` stores the last-used output name. On startup, clockie attempts to place itself on the configured output. If that output is not connected, it falls back to the compositor default.

```toml
[window]
output = "HDMI-A-1"
```

## How it works

The `wlr-layer-shell` protocol does not allow changing a surface's output after creation. When moving to a different output, clockie destroys the current layer surface and creates a new one bound to the target output, configured identically (size, anchor, margins, exclusive zone, keyboard interactivity).

Output adjacency is determined using the logical position and size reported by the compositor. The algorithm finds the output whose edge touches the current output's edge in the drag direction, with vertical/horizontal overlap.
