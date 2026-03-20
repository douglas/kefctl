# kefctl

TUI controller for KEF LSX II speakers.

Keyboard-driven terminal interface for managing KEF LSX II (W2 platform) speakers over their HTTP JSON API. Auto-discovers speakers via mDNS, provides real-time status updates, and supports scriptable CLI commands.

## Install

```sh
cargo install --path .
```

## Usage

```sh
# Launch TUI (auto-discovers speaker)
kefctl

# Connect to a specific speaker
kefctl --speaker 192.168.50.17

# Demo mode (no speaker needed)
kefctl --demo

# CLI commands (scriptable)
kefctl discover              # Find speakers on the network
kefctl status                # Print speaker status
kefctl source wifi           # Switch source
kefctl volume 30             # Set volume
```

## TUI Keybindings

| Key | Action |
|-----|--------|
| `q` / `Ctrl+c` | Quit |
| `Tab` / `Shift+Tab` | Next/prev panel |
| `j` / `k` | Move down/up |
| `h` / `l` | Focus sidebar/main panel |
| `Enter` | Select/confirm |
| `Esc` | Back to sidebar |
| `+` / `-` | Volume up/down |
| `m` | Toggle mute |
| `Space` | Play/pause |
| `n` / `p` | Next/previous track |
| `f` / `b` | Seek forward/backward 10s |
| `←` / `→` | Adjust value |

## Panels

- **Status** — Speaker info, now playing with progress bar, settings summary
- **Source** — Select input source (Wi-Fi, Bluetooth, USB, Optical, etc.)
- **EQ / DSP** — Treble, bass extension, desk/wall mode, subwoofer settings
- **Settings** — Cable mode, standby timeout, max volume, LED, startup tone
- **Network** — Connection status, discovered speakers

## Configuration

Optional config at `~/.config/kefctl/config.toml`:

```toml
[speaker]
ip = "192.168.50.17"
name = "Living Room"

[ui]
refresh_ms = 1000
```

## Themes

kefctl integrates with [Omarchy](https://github.com/basecamp/omarchy) for live theme switching. When Omarchy is installed, colors are read from `~/.config/omarchy/current/theme/colors.toml` at startup. Without Omarchy, a built-in default theme is used.

### Live reload

Send `SIGUSR1` to reload the theme without restarting:

```sh
pkill -USR1 kefctl
```

To auto-reload when Omarchy changes themes, add a hook:

```sh
mkdir -p ~/.config/omarchy/hooks/theme-set.d
cat > ~/.config/omarchy/hooks/theme-set.d/kefctl << 'EOF'
#!/bin/bash
pkill -USR1 kefctl 2>/dev/null || true
EOF
chmod +x ~/.config/omarchy/hooks/theme-set.d/kefctl
```

## Speaker Resolution

The speaker IP is resolved in this order:
1. `--speaker <ip>` flag
2. `speaker.ip` in config file
3. mDNS discovery (first KEF speaker found)

## License

MIT
