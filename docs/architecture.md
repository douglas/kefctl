# kefctl Architecture

## Overview

kefctl is a ~3500-line Rust TUI application that controls KEF W2-platform speakers over HTTP. It combines a Ratatui terminal UI with an async event loop for real-time speaker state updates.

## Data Flow

```
┌─────────────┐     HTTP JSON API      ┌──────────────┐
│             │ ◄────────────────────── │              │
│  KEF Speaker│     port 80            │   kefctl     │
│  (192.168.x)│ ──────────────────────► │              │
│             │  long-poll events       │              │
└─────────────┘                         └──────┬───────┘
       ▲                                       │
       │  mDNS _kef-info._tcp                  │
       └───────────────────────────────────────┘
```

### Startup sequence

1. Parse CLI args (clap) and load `~/.config/kefctl/config.toml`
2. Resolve speaker IP: `--speaker` flag → config file → cached IP → mDNS discovery
3. `KefClient::fetch_full_state()` — parallel HTTP GETs for all settings
4. Initialize `App` with `SpeakerState` + `Theme::load()`
5. Enter TUI event loop

### Event loop (main.rs → run_tui_loop)

```
┌──────────────────────────────────────────────────────┐
│                    Event Loop                         │
│                                                       │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────┐ │
│  │  Terminal    │  │  Speaker     │  │  SIGUSR1    │ │
│  │  Events     │  │  Poll Task   │  │  Listener   │ │
│  │  (crossterm)│  │  (HTTP long  │  │  (theme     │ │
│  │             │  │   poll)      │  │   reload)   │ │
│  └──────┬──────┘  └──────┬───────┘  └──────┬──────┘ │
│         │                │                  │        │
│         └────────┬───────┴──────────────────┘        │
│                  ▼                                    │
│         mpsc::UnboundedChannel<Event>                │
│                  │                                    │
│                  ▼                                    │
│         ┌────────────────┐                           │
│         │  Event Handler │                           │
│         │  Key → Action  │                           │
│         │  Tick → update │                           │
│         │  Speaker → sync│                           │
│         │  Theme → reload│                           │
│         └────────┬───────┘                           │
│                  │                                    │
│                  ▼                                    │
│         ┌────────────────┐     ┌──────────────┐     │
│         │  App::handle   │────►│  dispatch     │     │
│         │  (optimistic   │     │  (async HTTP  │     │
│         │   UI update)   │     │   w/ error tx)│     │
│         └────────────────┘     └──────────────┘     │
└──────────────────────────────────────────────────────┘
```

`dispatch_action` spawns async HTTP requests via `tokio::spawn`. On failure, errors are sent back through the event channel as `Event::SpeakerError`, surfacing as TUI notifications. Graceful shutdown uses `CancellationToken` from `tokio-util` to cancel all spawned tasks (speaker poll loop, SIGUSR1 listener).

## Module Map

### Core

| Module | Responsibility |
|--------|---------------|
| `main.rs` | CLI dispatch, TUI loop, async action dispatch |
| `app.rs` | All application state: `App`, `SpeakerState`, `Panel`, `Focus`, `Action` enum, keyboard handling |
| `event.rs` | Async event multiplexer: terminal input, tick timer, speaker polling, SIGUSR1 |
| `tui.rs` | Terminal setup/teardown (alternate screen, raw mode) |
| `cli.rs` | CLI argument parsing (clap derive) |
| `config.rs` | TOML config file loader |
| `discovery.rs` | mDNS speaker discovery via `_kef-info._tcp.local.` |
| `error.rs` | `KefError` enum (network, API, json, type mismatch, discovery, config) |

### KEF API Client (`kef_api/`)

| Module | Responsibility |
|--------|---------------|
| `mod.rs` | `KefClient` struct, `get_data`/`set_data` core methods, `fetch_full_state`, `extract_string`/`extract_i32`/`extract_bool` pure functions, `sanitize()` for network strings |
| `types.rs` | `ApiValue` tagged union serde, `Source`, `StandbyMode`, `CableMode`, `WakeUpSource`, `EqProfile` |
| `volume.rs` | `get_volume`, `set_volume`, `get_max_volume`, `get_mute`, `set_mute` |
| `source.rs` | `get_source`, `set_source` |
| `settings.rs` | `get/set_standby_mode`, `get/set_cable_mode`, `get/set_wake_up_source`, LED, startup tone, app analytics, device name |
| `paths.rs` | API path string constants |
| `events.rs` | `subscribe`, `poll_events` (long-poll), `unsubscribe` |

### UI (`ui/`)

| Module | Responsibility |
|--------|---------------|
| `mod.rs` | Top-level layout (sidebar + main), footer bar, notification overlay |
| `theme.rs` | `Theme` struct (13 color fields), Omarchy loader, `block()` / `info_row()` / `section_block()` helpers |
| `sidebar.rs` | Panel navigation list with focus highlighting |
| `status.rs` | Speaker info, settings summary; inline device name editor (`e` key) |
| `source.rs` | Input source selector with active marker |
| `eq.rs` | EQ parameter editor (treble, bass ext, desk/wall mode, sub, phase) |
| `settings.rs` | Settings editor (standby, max vol, LED, startup tone, cable mode, wake-up source, app analytics) |
| `network.rs` | Connection status + discovered speakers list |
| `help.rs` | Floating keybindings overlay |

## Key Patterns

### ApiValue tagged union

The KEF API wraps all values in a type-tagged JSON object:

```json
{"type": "i32_", "i32_": 50}
{"type": "kefPhysicalSource", "kefPhysicalSource": "wifi"}
{"type": "kefStandbyMode", "kefStandbyMode": "standby_30mins"}
```

This maps to `ApiValue` enum with `#[serde(tag = "type")]` and per-variant rename.

### Optimistic UI updates

When the user presses a key (e.g., volume up), the app:
1. Updates `SpeakerState` immediately (instant UI feedback)
2. Fires an async HTTP request via `dispatch_action` (tokio::spawn, errors sent back via event channel)
3. Speaker poll loop will eventually confirm or revert the state

### Theme system

All colors flow through `app.theme`. The `Theme::block(title, focused)` helper eliminates duplicated border construction across panels. `info_row()` and `section_block()` provide consistent styling for labeled key-value rows and sub-sections. SIGUSR1 triggers `Theme::load()` which re-reads Omarchy colors.

### Speaker event polling

1. Subscribe to paths via `GET /api/event/modifyQueue`
2. Long-poll via `GET /api/event/pollQueue` (30s server timeout, 60s client timeout)
3. On event: re-fetch full state and update `App`
4. On timeout: re-poll silently (no error)
5. On error: notify user, wait 5s, re-subscribe

## Security Model

kefctl communicates over **plaintext HTTP** on the local network — a hardware constraint of the KEF speaker API. The threat model is a hostile device on the same LAN impersonating a speaker.

### Hardening measures

| Concern | Mitigation |
|---------|-----------|
| Unsafe Rust | `#![forbid(unsafe_code)]` — compiler-enforced, cannot be overridden |
| SSRF via redirect | `redirect::Policy::none()` on all reqwest clients |
| Memory exhaustion | `MAX_RESPONSE_BYTES = 64KB` — bytes checked before deserialization |
| Terminal injection | `sanitize()` strips control chars (incl. DEL 0x7F) from all API strings and error bodies |
| mDNS name injection | `sanitize_name()` in `discovery.rs` strips control chars from mDNS names |
| Symlink attacks | Atomic write (temp+rename) for all state files; no `/tmp` fallback |
| File disclosure | State/log files: `0o600`; state directories: `0o700` |
| Supply chain | `cargo-audit` + `cargo-deny` in CI; policy in `deny.toml` |

### Trust boundaries

- **Trusted:** CLI flags, config file (`~/.config/kefctl/config.toml`), local state files
- **Untrusted:** All data received from the speaker API and mDNS — sanitized before display or storage

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui` + `crossterm` | Terminal UI framework |
| `tokio` | Async runtime (scoped features, not `"full"`) |
| `tokio-util` | `CancellationToken` for graceful shutdown |
| `futures` | Stream combinators for async event processing |
| `reqwest` | HTTP client for KEF API |
| `serde` + `serde_json` | JSON serialization (ApiValue tagged union) |
| `toml` | Config file + Omarchy theme parsing |
| `clap` | CLI argument parsing |
| `mdns-sd` | mDNS/DNS-SD speaker discovery |
| `dirs` | XDG directory resolution |
| `tracing` + `tracing-subscriber` | File-based logging |
| `thiserror` | Error enum derive |
