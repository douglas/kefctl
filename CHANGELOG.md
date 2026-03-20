# Changelog

All notable changes to kefctl will be documented in this file.

## [0.1.0] — 2026-03-20

Initial release.

### Features

- **TUI** — Keyboard-driven terminal interface with sidebar navigation and 5 panels (Status, Source, EQ/DSP, Settings, Network)
- **CLI commands** — `discover`, `status`, `source`, `volume` for scripting
- **mDNS discovery** — Auto-finds KEF speakers via `_kef-info._tcp.local.`
- **Real-time updates** — Long-poll event subscription keeps UI in sync with speaker state
- **Help overlay** — Press `?` for keybindings reference
- **Omarchy theme integration** — Reads colors from `~/.config/omarchy/current/theme/colors.toml`, live reload via `SIGUSR1`
- **Focus-based borders** — Thick colored borders on focused panel, dim on unfocused
- **Styled footer** — Key badges, connection indicator, speaker name, active panel
- **Demo mode** — `--demo` flag for development without a speaker
- **File logging** — Logs to `~/.local/state/kefctl/kefctl.log`

### Panels

- **Status** — Speaker info, settings summary, now playing with progress bar and controls hint
- **Source** — Select input (Wi-Fi, Bluetooth, USB, TV, Optical, Coaxial, Analog) with active marker
- **EQ / DSP** — Edit treble, bass extension, desk/wall mode, subwoofer, phase correction
- **Settings** — Cycle cable mode, standby timeout, max volume, front LED, startup tone
- **Network** — Connection status, discovered speakers list

### KEF API

- HTTP JSON client for KEF W2 platform (LSX II, LS50 Wireless II, LS60 Wireless)
- Tagged union serde for `ApiValue` (`i32_`, `string_`, `bool_`, `kefPhysicalSource`, `kefStandbyMode`, `kefCableMode`)
- Optimistic UI updates with async fire-and-forget API dispatch
- Event subscribe/poll with graceful timeout handling

### Configuration

- Optional `~/.config/kefctl/config.toml` for speaker IP and refresh rate
- Speaker resolution: `--speaker` flag → config → mDNS discovery
