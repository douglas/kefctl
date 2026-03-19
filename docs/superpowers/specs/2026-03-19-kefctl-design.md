# kefctl — KEF LSX II TUI Controller

## Overview

A Rust TUI application for fully managing KEF LSX II (W2 platform) speakers from the terminal. Sidebar navigation layout inspired by Impala, built with Ratatui. Communicates with the speaker over its HTTP JSON API. Auto-discovers speakers via mDNS with config file fallback.

## Goals

- Full control of KEF LSX II from the terminal: source, volume, EQ, playback, settings
- Real-time status updates via event long-polling
- Clean, keyboard-driven Impala-style sidebar layout
- Single binary, no runtime dependencies
- Arch Linux / Omarchy first-class support

## Non-Goals

- Multi-room / speaker grouping (defer to v2)
- Streaming media to the speaker (use existing tools)
- GUI or web interface

## Architecture

```
┌──────────────┐     HTTP/JSON      ┌────────────┐
│   kefctl     │ ←────────────────→ │  KEF LSX   │
│  (Ratatui)   │  /api/getData      │    II       │
│              │  /api/setData      │            │
│              │  /api/event/poll   │            │
└──────────────┘                    └────────────┘
```

### Module Structure

```
src/
├── main.rs              # Entry point, arg parsing, tokio runtime
├── app.rs               # Application state and state transitions
├── event.rs             # Event loop: terminal keys + speaker poll events
├── ui/
│   ├── mod.rs           # Top-level render: sidebar + main panel
│   ├── sidebar.rs       # Sidebar widget
│   ├── status.rs        # Status panel (home screen)
│   ├── source.rs        # Source selector panel
│   ├── eq.rs            # EQ/DSP panel
│   ├── settings.rs      # Settings panel
│   └── network.rs       # Network/discovery panel
├── kef_api/
│   ├── mod.rs           # Client struct, connection management
│   ├── types.rs         # Request/response types (serde)
│   ├── playback.rs      # Play, pause, next, prev, seek
│   ├── volume.rs        # Volume, mute, max volume
│   ├── source.rs        # Source get/set
│   ├── settings.rs      # Device settings (standby, cable, LED, etc.)
│   ├── eq.rs            # EQ profiles, DSP params
│   └── events.rs        # Event polling (pollQueue)
├── discovery.rs         # mDNS speaker discovery
└── config.rs            # Config file parsing (~/.config/kefctl/config.toml)
```

## Components

### 1. `kef_api` — HTTP Client Layer

Typed async HTTP client wrapping the KEF speaker's REST API. All methods return `Result<T, KefError>`.

```rust
enum KefError {
    Network(reqwest::Error),
    Api { status: u16, message: String },
    Timeout,
    Discovery(String),
    Config(toml::de::Error),
}
```

**Base URL:** `http://<speaker-ip>/api/`

**Core endpoints used:**

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/getData` | GET/POST | Read speaker state (source, volume, settings, player data) |
| `/api/setData` | POST | Write speaker state (change source, volume, settings) |
| `/api/getRows` | GET | Read list data (group members — v2 only, not used in v1) |
| `/api/event/modifyQueue` | GET | Register/unsubscribe event subscriptions, returns queueId |
| `/api/event/pollQueue` | GET | Long-poll for real-time state changes (requires queueId from modifyQueue) |

**API paths (from KEF HTTP API):**

- `player:volume` — current volume (i32, 0–100)
- `player:player/data` — now playing info (artist, track, duration, position)
- `player:player/data/playTime` — playback position
- `player:player/control` — playback control (play, pause, next, previous, seekTime)
- `settings:/kef/play/physicalSource` — active source (usb, wifi, bluetooth, tv, optical, coaxial, analog, standby)
- `settings:/kef/host/speakerStatus` — powerOn or standby
- `settings:/kef/host/cableMode` — wired or wireless
- `settings:/kef/host/maximumVolume` — max volume limit
- `settings:/kef/host/standbyMode` — auto-standby timeout
- `settings:/kef/host/startupTone` — enable/disable
- `settings:/kef/host/disableFrontStandbyLED` — LED control
- `settings:/kef/host/wakeUpSource` — wake-up trigger
- `settings:/mediaPlayer/mute` — mute state
- `settings:/deviceName` — speaker name
- `settings:/releasetext` — firmware version
- `settings:/system/primaryMacAddress` — MAC address
- `kef:eqProfile/v2` — active EQ profile and DSP parameters

**Request/Response Formats:**

`getData` uses query params: `GET /api/getData?path=settings%3A%2Fkef%2Fhost%2FcableMode&roles=value`
Returns: `[{"kefCableMode": "wired", "type": "kefCableMode"}]`

`setData` uses POST with typed value wrappers:
```json
{
  "path": "player:volume",
  "roles": "value",
  "value": { "type": "i32_", "i32_": 50 }
}
```
Value types: `i32_`, `i64_`, `string_`, `bool_`, `kefPhysicalSource`, `kefSpeakerStatus`, `kefCableMode`

**Event registration flow:**
1. `GET /api/event/modifyQueue?subscribe=[paths]&queueId=` → returns `{"queueId": "<uuid>"}`
2. `GET /api/event/pollQueue?queueId=<uuid>&timeout=5000` → returns changed values or empty on timeout

### 2. `discovery` — mDNS Speaker Discovery

Uses `mdns-sd` crate to browse for `_http._tcp.local.` services, filtering results where the service name contains "KEF" (case-insensitive). The exact filter heuristic may need refinement during implementation by inspecting mDNS TXT records on a live network. Returns list of `(name, ip, port)` tuples. Times out after 5 seconds.

Falls back to `~/.config/kefctl/config.toml` if no speakers found or if a static IP is configured.

### 3. `app` — Application State

```rust
struct App {
    speaker: SpeakerState,      // Current speaker state (all fields)
    panel: Panel,               // Active sidebar panel
    connection: ConnectionState, // Connected / Disconnected / Connecting
    // Per-panel state
    source_list: ListState,     // Source selector cursor
    eq_focus: usize,            // Which EQ parameter row is focused
    settings_focus: usize,      // Which setting row is focused
    network_speakers: Vec<Speaker>, // Discovered speakers
}

enum Panel {
    Status,
    Source,
    Eq,
    Settings,
    Network,
}

struct SpeakerState {
    name: String,
    model: String,              // Parsed from settings:/deviceName (e.g., "LSX II" from "LSX II")
    firmware: String,
    ip: IpAddr,
    mac: String,
    source: Source,
    volume: i32,            // API returns i32, clamped to 0..=max_volume
    muted: bool,
    cable_mode: CableMode,
    standby_mode: StandbyMode,
    max_volume: i32,
    front_led: bool,
    startup_tone: bool,
    eq_profile: String,
    // Now playing
    artist: Option<String>,
    track: Option<String>,
    duration: Option<u32>,
    position: Option<u32>,
    playing: bool,
}
```

### 4. `ui` — Ratatui Rendering

**Layout:** Two-column split. Left column is fixed-width sidebar (15 chars). Right column is the main panel, rendered by the active panel's function.

**Sidebar:** List of panel names. Active panel highlighted. `j/k` or `↑/↓` to navigate, `Enter` or `l/→` to focus main panel.

**Main panels:**

- **Status** — Speaker info block, now playing with progress bar and playback controls, key settings summary
- **Source** — Selectable list of sources, active source marked, Enter to switch
- **EQ** — Profile selector, treble dB adjustment, bass extension (less/standard/more), desk mode toggle + dB, wall mode toggle + dB, subwoofer settings (gain/polarity/crossover). Arrow keys to navigate and adjust values.
- **Settings** — Grouped settings with inline cycling (◂ ▸), Enter to confirm changes
- **Network** — Discovered speakers list, connection status, manual IP entry

**Keybindings (global):**

| Key | Action |
|-----|--------|
| `q` / `Ctrl+c` | Quit |
| `Tab` / `Shift+Tab` | Next/prev sidebar panel |
| `j` / `↓` | Move down in list |
| `k` / `↑` | Move up in list |
| `h` | Focus sidebar (from main panel) |
| `l` | Focus main panel (from sidebar) |
| `←` / `→` | Decrease / increase value (when editing a control) |
| `Esc` | Cancel edit / return to sidebar |
| `Enter` | Select / confirm |
| `m` | Toggle mute |
| `+` / `-` | Volume up / down |
| `Space` | Play / pause |
| `n` | Next track |
| `p` | Previous track |
| `f` / `b` | Seek forward / backward 10s |

### 5. `event` — Event Loop

Merges two async streams:

1. **Terminal events** — `crossterm::event::EventStream`, key presses and resize
2. **Speaker events** — register via `/api/event/modifyQueue` then long-poll `/api/event/pollQueue` with 5s timeout, reconnect on failure

Uses `tokio::select!` to process whichever fires first. Terminal events trigger state transitions and API calls. Speaker events update `SpeakerState` and trigger re-render.

Tick timer (1s) updates playback progress bar between poll events. Poll events with `player:player/data/playTime` are authoritative and always overwrite the locally-ticked position.

**Event subscriptions:** Register for `player:volume`, `player:player/data`, `settings:/kef/play/physicalSource`, `settings:/kef/host/speakerStatus`, `settings:/mediaPlayer/mute`, `settings:/kef/host/cableMode`.

**Shutdown:** On quit, drop the connection. The speaker cleans up stale event subscriptions automatically.

### 6. `config` — Configuration

`~/.config/kefctl/config.toml`:

```toml
[speaker]
ip = "192.168.50.17"       # Optional: skip discovery, connect directly
name = "LSX II"             # Optional: display name override

[ui]
refresh_ms = 1000            # Tick interval for progress bar updates
```

Config is optional. Without it, kefctl discovers speakers via mDNS and uses the first one found.

## Data Flow

### Startup Sequence

1. Parse CLI args (`--speaker <ip>` override)
2. Load config from `~/.config/kefctl/config.toml`
3. If no IP from args or config: run mDNS discovery (5s timeout)
4. If multiple speakers found: show Network panel for selection
5. Fetch full speaker state via batch getData calls
6. Start event poll loop
7. Render initial UI on Status panel

### Runtime Loop

```
Terminal Event ──→ match action ──→ API call (setData) ──→ optimistic UI update
                                                              │
Speaker Poll  ──→ parse event ──→ update SpeakerState ──→ re-render
                                                              │
Tick Timer    ──→ increment playback position ──→ re-render progress bar
```

### User Action Example (Volume Up)

1. User presses `+`
2. Event loop matches to `Action::VolumeUp`
3. `app.speaker.volume = min(volume + 1, max_volume)` (optimistic)
4. Spawn `kef_api.set_volume(new_volume)` task (rapid presses debounced: coalesce within 50ms, cancel in-flight request for same path)
5. UI re-renders with new volume
6. If API call fails: revert optimistic update, show inline error notification (auto-dismiss 3s)
7. Otherwise, next poll event confirms the value

## Error Handling

- **Speaker unreachable** — `connection` state goes to `Disconnected`, status bar shows warning, auto-retry every 5 seconds
- **API call fails** — inline notification in active panel (e.g., "Failed to change source"), non-blocking, auto-dismiss after 3s
- **Discovery finds nothing** — show Network panel with manual IP entry prompt
- **Invalid config** — print error to stderr and exit with helpful message

## Dependencies

```toml
[dependencies]
ratatui = "0.29"
crossterm = "0.28"
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
mdns-sd = "0.11"
toml = "0.8"
clap = { version = "4", features = ["derive"] }
dirs = "6"
```

## CLI Interface

```
kefctl                       # Launch TUI, auto-discover speaker
kefctl --speaker 192.168.50.17  # Connect to specific IP
kefctl discover              # List speakers on network and exit
kefctl status                # Print current speaker status and exit
kefctl source usb            # Switch source and exit (scriptable)
kefctl volume 50             # Set volume and exit (scriptable)
```

Non-interactive commands (`discover`, `status`, `source`, `volume`) enable scripting and integration with other tools (e.g., keybindings, waybar modules).

Non-interactive commands resolve the speaker using: `--speaker` flag > config file IP > mDNS discovery. If mDNS finds multiple speakers and no `--speaker`/config is set, exit with error listing discovered speakers.

## Success Criteria

1. Launch kefctl → auto-discovers KEF LSX II → shows Status panel with live data
2. Navigate all 5 sidebar panels with keyboard
3. Switch source → speaker changes input within 1 second
4. Adjust volume → smooth, responsive, real-time feedback
5. EQ adjustment → visual sliders update, speaker applies changes
6. Settings changes → confirmed by re-reading from speaker
7. Disconnect speaker → graceful degradation, auto-reconnect
8. Non-interactive CLI commands work for scripting
