# Changelog

All notable changes to kefctl will be documented in this file.

## [Unreleased]

### Changed

- Removed unused `color-eyre` dependency
- Scoped tokio features (`rt-multi-thread`, `sync`, `time`, `signal`, `macros`) instead of `"full"`
- API action errors now surface as TUI notifications via event channel (previously only logged)
- Extracted pure `extract_string`/`extract_i32`/`extract_bool` functions in `kef_api/mod.rs` for testability
- Fixed tilde `PathBuf` fallback in config (`~/` doesn't expand in `PathBuf::from`)

### Added

- GitHub Actions CI: clippy (`-D warnings`), test, and release build
- `[profile.release]` with LTO, strip, `codegen-units = 1`
- Graceful async shutdown via `CancellationToken` (`tokio-util`)
- `KefError::TypeMismatch` variant for typed getter validation
- `KefError::Config` variant for TOML parse errors
- `kef_api/paths.rs` — API path string constants (replaces magic strings)
- `theme.info_row()` and `theme.section_block()` UI helpers
- 41 new tests (48 → 89): app state machine, UI rendering (TestBackend), error Display formats, API extraction, EqProfile serde, I64 roundtrip, config parsing

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
- Optimistic UI updates with async API dispatch
- Event subscribe/poll with graceful timeout handling

### Configuration

- Optional `~/.config/kefctl/config.toml` for speaker IP and refresh rate
- Speaker resolution: `--speaker` flag → config → mDNS discovery
