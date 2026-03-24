# Changelog

All notable changes to kefctl will be documented in this file.

## [0.3.0] — 2026-03-24

### Added

- `kefctl toggle` — wake speaker to last-used source or send to standby
- `kefctl waybar` — JSON status output for waybar custom module
- `kefctl mute [on|off]` — scriptable mute control; omit argument to toggle
- Last-used source persistence (`~/.local/state/kefctl/last_source`)
- `default_source` config option as fallback for `kefctl toggle`
- Auto-detect speaker model from firmware string (LSX II, LS50 Wireless II, LS60 Wireless)
- Event subscriptions for standby mode, max volume, and EQ profile (8 paths total, was 5)
- Real EQ/DSP data from `kef:eqProfile/v2` API endpoint (22 fields)
- `unwrap_or_exit()` helper in main.rs to eliminate duplicate error-exit boilerplate
- AUR auto-publish workflow (`.github/workflows/aur-publish.yml`) — triggered on `v*` tags
- `aur/PKGBUILD` as authoritative AUR package source in this repo

### Changed

- EQ/DSP panel shows real speaker data (was hardcoded defaults)
- Settings panel: cable mode moved to display-only info section
- `load_cached_ip()` validates and returns `IpAddr` directly (was `Option<String>`)
- Cargo.toml and PKGBUILD description updated to mention all W2 models

### Removed

- `SetCableMode` action (was a no-op)
- EQ value adjustment handlers (panel is read-only until write API is implemented)
- `SubPolarity` enum, `BassExtension` cycling methods (unused)
- `/tmp` fallback for state/log directories — skip and warn instead

### Security

- JSON injection fix: waybar output uses `serde_json::json!()` instead of string interpolation
- HTTP redirects disabled (`redirect::Policy::none()`) on both reqwest clients (SSRF prevention)
- Network-sourced strings sanitized: control characters stripped from API strings and mDNS names
- State files use atomic writes (write-then-rename) with 0o600 permissions
- Log file opened with mode 0o600
- Cached IPs validated as `IpAddr` on load
- Updated rustls-webpki to resolve RUSTSEC-2026-0049

## [0.2.0] — 2026-03-20

### Changed

- Removed unused `color-eyre` dependency
- Scoped tokio features (`rt-multi-thread`, `sync`, `time`, `signal`, `macros`) instead of `"full"`
- API action errors now surface as TUI notifications via event channel (previously only logged)
- Extracted pure `extract_string`/`extract_i32`/`extract_bool` functions in `kef_api/mod.rs` for testability
- Fixed tilde `PathBuf` fallback in config (`~/` doesn't expand in `PathBuf::from`)

### Removed

- Now-playing display, progress bar, and playback controls (Space/n/p/f/b) — kefctl is a speaker settings app, not a media player. Playback is handled by Spotify/Roon/AirPlay.
- `kef_api/playback.rs` module, `PLAYER_DATA`/`PLAYER_CONTROL` API paths
- `artist`, `track`, `duration`, `position`, `playing` fields from `SpeakerState`
- `progress_filled`/`progress_empty` from Theme (no longer needed without progress bar)

### Added

- Cached speaker IP: after a successful connection, the speaker IP is saved to `~/.local/state/kefctl/last_speaker` and tried first on next launch, skipping 5-second mDNS discovery
- GitHub Actions CI: clippy (`-D warnings`), test, and release build
- `[profile.release]` with LTO, strip, `codegen-units = 1`
- Graceful async shutdown via `CancellationToken` (`tokio-util`)
- `KefError::TypeMismatch` variant for typed getter validation
- `KefError::Config` variant for TOML parse errors
- `kef_api/paths.rs` — API path string constants (replaces magic strings)
- `theme.info_row()` and `theme.section_block()` UI helpers
- 47 new tests (48 → 95): app state machine, UI rendering (TestBackend), error Display formats, API extraction, EqProfile serde, I64 roundtrip, config parsing, insta snapshot tests for all panels
- `#![deny(unsafe_code)]` — no unsafe in the codebase
- `#[must_use]` on `handle_key()` to prevent dropped actions
- Cross-platform `KeyEventKind::Press` filter in event handler
- All `pub` items tightened to `pub(crate)` (single-binary crate)
- Module-level `//!` doc comments on all 25 `.rs` files
- `rust-version = "1.86.0"` MSRV in Cargo.toml
- `ValueEnum` for CLI source argument (replaces manual string matching)
- `KEFCTL_LOG` environment variable for runtime log level control via `EnvFilter`
- `#[tracing::instrument]` spans on key async API methods
- 2-second `connect_timeout` on reqwest clients for faster failure detection
- `insta` dev-dependency for visual regression snapshot tests
- `cargo doc --no-deps` step in CI
- Cargo.toml metadata: `license`, `repository`, `keywords`, `categories`
- Partial Omarchy theme loading — missing colors fall back to defaults instead of failing entirely
- Extracted keybinding hint constants (`HINT_ADJUST`, `HINT_CYCLE`) in UI module

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

- **Status** — Speaker info, settings summary
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
