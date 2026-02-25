# IPC protocol

For programmatic control beyond `clockiectl`, you can send JSON commands directly to the Unix socket.

**Socket location:** `$XDG_RUNTIME_DIR/clockie.sock` (fallback: `/tmp/clockie-$UID.sock`)

**Protocol:** Send a single JSON object followed by a newline (`\n`). Read one JSON line back as the response.

## Commands

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
| Move to output | `{"cmd": "move-to-output", "name": "HDMI-A-1"}` |
| Reload config | `{"cmd": "reload-config"}` |
| Get state | `{"cmd": "get-state"}` |
| Quit | `{"cmd": "quit"}` |

The `move-to-output` command also accepts `"next"` and `"prev"` as the name to cycle through outputs.

## Responses

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
  "locked": false,
  "output": "eDP-1"
}
```

## Example with socat

```sh
echo '{"cmd":"get-state"}' | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/clockie.sock
```
