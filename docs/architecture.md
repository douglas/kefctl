# kefctl Architecture

## Overview

kefctl is a ~3000-line Rust TUI application that controls KEF W2-platform speakers over HTTP. It combines a Ratatui terminal UI with an async event loop for real-time speaker state updates.

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
| `error.rs` | `KefError` enum (network, API, type mismatch, discovery, config) |

### KEF API Client (`kef_api/`)

| Module | Responsibility |
|--------|---------------|
| `mod.rs` | `KefClient` struct, `get_data`/`set_data` core methods, `fetch_full_state`, `extract_string`/`extract_i32`/`extract_bool` pure functions |
| `types.rs` | `ApiValue` tagged union serde, `Source`, `StandbyMode`, `CableMode`, `EqProfile` |
| `volume.rs` | `get_volume`, `set_volume`, `get_max_volume`, `get_mute`, `set_mute` |
| `source.rs` | `get_source`, `set_source` |
| `playback.rs` | `play`, `pause`, `next_track`, `previous_track`, `seek` |
| `settings.rs` | `get/set_standby_mode`, `get/set_cable_mode`, LED, startup tone |
| `paths.rs` | API path string constants |
| `events.rs` | `subscribe`, `poll_events` (long-poll), `unsubscribe` |

### UI (`ui/`)

| Module | Responsibility |
|--------|---------------|
| `mod.rs` | Top-level layout (sidebar + main), footer bar, notification overlay |
| `theme.rs` | `Theme` struct (13 color fields), Omarchy loader, `block()` / `info_row()` / `section_block()` helpers |
| `sidebar.rs` | Panel navigation list with focus highlighting |
| `status.rs` | Speaker info, settings summary, now playing + progress bar |
| `source.rs` | Input source selector with active marker |
| `eq.rs` | EQ parameter editor (treble, bass ext, desk/wall mode, sub, phase) |
| `settings.rs` | Settings editor (cable, standby, max vol, LED, startup tone) |
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
