# Level Up: From Ruby to kefctl

A guided tour for Ruby developers who want to contribute to kefctl. You don't need to learn all of Rust — just the parts this project uses.

## 1. Rust Basics (the Ruby translation)

If you know Ruby, you already know most of the concepts. The syntax is different but the ideas map directly.

### Variables and types

```ruby
# Ruby
name = "kefctl"
volume = 50
muted = false
```

```rust
// Rust — types are inferred or declared
let name = "kefctl";          // &str (like a frozen string)
let name = String::from("x"); // String (like a regular string)
let volume: i32 = 50;         // explicit type
let muted = false;
let mut volume = 50;          // `mut` = mutable (default is immutable!)
volume += 1;                  // ok because `mut`
```

**Key difference:** Variables are immutable by default. You opt in to mutation with `mut`. This is the opposite of Ruby where everything is mutable.

### Structs (like Ruby classes, but no inheritance)

```ruby
# Ruby
class Speaker
  attr_accessor :name, :volume, :muted
  def initialize(name, volume, muted)
    @name = name
    @volume = volume
    @muted = muted
  end
end
```

```rust
// Rust
struct Speaker {
    name: String,
    volume: i32,
    muted: bool,
}

impl Speaker {
    fn new(name: String, volume: i32) -> Self {
        Self { name, volume, muted: false }
    }

    fn display_name(&self) -> &str {  // &self = like Ruby's self
        &self.name
    }

    fn set_volume(&mut self, vol: i32) {  // &mut self = needs mutable access
        self.volume = vol;
    }
}
```

**In kefctl:** `App` struct in `app.rs` holds all state. `SpeakerState` holds speaker data. Methods are in `impl` blocks.

### Enums (like Ruby symbols on steroids)

```ruby
# Ruby
SOURCES = [:wifi, :bluetooth, :usb, :tv]
```

```rust
// Rust — enums can hold data!
enum Source {
    Wifi,
    Bluetooth,
    Usb,
    Tv,
}

// Pattern matching (like Ruby case, but exhaustive)
match source {
    Source::Wifi => "Wi-Fi",
    Source::Bluetooth => "Bluetooth",
    Source::Usb => "USB",
    Source::Tv => "TV",
}
// ^ compiler ERROR if you forget a variant. This catches bugs.
```

**In kefctl:** `Panel`, `Focus`, `Action`, `Source`, `ConnectionState` are all enums. The `match` keyword is used everywhere — the compiler forces you to handle every case.

### Option and Result (no nil, no exceptions)

```ruby
# Ruby — anything can be nil, exceptions can happen anywhere
track = speaker.now_playing  # might be nil
data = api.fetch!            # might raise
```

```rust
// Rust — the type system encodes "might not exist" and "might fail"
let track: Option<String> = Some("Says".to_string());  // or None
let track: Option<String> = None;

// You must handle both cases
match track {
    Some(t) => println!("Playing: {t}"),
    None => println!("No track"),
}
// Shorthand:
let name = track.unwrap_or("No track".to_string());
let name = track.as_deref().unwrap_or("No track");  // common in kefctl

// Results for operations that can fail
let volume: Result<i32, KefError> = client.get_volume().await;
match volume {
    Ok(v) => println!("Volume: {v}"),
    Err(e) => println!("Error: {e}"),
}
// Shorthand: the ? operator (like Ruby's &. but for errors)
let volume = client.get_volume().await?;  // returns Err early if it fails
```

**In kefctl:** `Option` is used for track/artist/duration (might not be playing). `Result` + `?` is used for all API calls. `KefError` in `error.rs` is the error type.

### Ownership (the one new concept)

This is the thing Ruby doesn't have. Values have exactly one owner. When the owner goes out of scope, the value is dropped (like Ruby's GC but deterministic).

```rust
let name = String::from("LSX II");
let other = name;        // name is MOVED to other
// println!("{name}");   // ERROR: name was moved

// Borrowing: lend access without moving
let name = String::from("LSX II");
let r = &name;           // immutable borrow (can have many)
println!("{r}");         // fine
println!("{name}");      // still fine — name wasn't moved

fn print_name(s: &str) { // takes a borrow, doesn't own it
    println!("{s}");
}
```

**In kefctl:** You'll see `&self`, `&App`, `&str` everywhere — these are borrows. The `app` variable in the TUI loop owns everything. UI functions borrow it with `&App` or `&mut App`.

**Practical rule:** If the compiler complains about ownership, try adding `&` (borrow), `.clone()` (copy), or `.to_string()` (convert). The compiler errors are very helpful — read them.

## 2. Async Rust (like Ruby threads but safe)

kefctl uses `tokio` for async, similar to Ruby's async/await or Ractors.

```ruby
# Ruby (conceptual)
result = await fetch_volume()
```

```rust
// Rust
let result = client.get_volume().await;  // .await pauses until done

// Running things in parallel (like Ruby Thread.new)
let (volume, source) = tokio::try_join!(
    client.get_volume(),
    client.get_source(),
)?;  // runs both simultaneously, fails if either fails

// Fire and forget (like Ruby Thread.new without join)
tokio::spawn(async move {
    let _ = client.set_volume(50).await;
});
```

**In kefctl:** `fetch_full_state()` uses `tokio::try_join!` to fetch all settings in parallel. Action dispatch uses `tokio::spawn` with error feedback through the event channel. The event loop in `event.rs` uses `tokio::select!` to wait on multiple event sources.

### Graceful shutdown with CancellationToken

When the user quits, all spawned tasks need to stop cleanly. kefctl uses `CancellationToken` from `tokio-util`:

```rust
use tokio_util::sync::CancellationToken;

let cancel = CancellationToken::new();
let token = cancel.clone();  // clone for each spawned task

tokio::spawn(async move {
    loop {
        tokio::select! {
            () = token.cancelled() => break,  // stop when cancelled
            _ = do_work() => { /* process result */ }
        }
    }
});

// Later, to shut down all tasks:
cancel.cancel();
```

**In kefctl:** `EventHandler` in `event.rs` creates a `CancellationToken` and shares clones with the terminal event task, speaker poll task, and SIGUSR1 listener. Calling `events.shutdown()` cancels all three at once. This is safer than dropping channels or using atomic bools — the `select!` ensures tasks check for cancellation at every await point.

## 3. Serde (JSON serialization)

Like Ruby's `JSON.parse`/`to_json` but with type safety.

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]  // auto-generates JSON conversion
struct Speaker {
    name: String,
    volume: i32,
}

// Deserialize (parse)
let s: Speaker = serde_json::from_str(r#"{"name":"LSX II","volume":50}"#)?;

// Serialize (generate)
let json = serde_json::to_string(&s)?;
```

**In kefctl:** The `ApiValue` enum in `types.rs` is the trickiest serde code. The KEF API uses tagged unions:

```json
{"type": "i32_", "i32_": 50}
```

This maps to:

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]  // use "type" field to pick variant
enum ApiValue {
    #[serde(rename = "i32_")]
    I32 {
        #[serde(rename = "i32_")]
        value: i32,
    },
    // ...
}
```

The `#[serde(rename = "...")]` attributes control the JSON field names. If you add a new API type, follow this pattern.

## 4. Ratatui (the TUI framework)

Ratatui is an immediate-mode UI library. Every frame, you describe what to draw. There's no retained widget tree (unlike HTML/React).

### The render cycle

```rust
// Every frame (~1 second):
terminal.draw(|frame| {
    // Describe the entire screen from scratch
    let area = frame.area();
    frame.render_widget(some_widget, area);
})?;
```

### Layout

```rust
use ratatui::layout::{Layout, Constraint};

// Split area vertically into 3 chunks
let chunks = Layout::vertical([
    Constraint::Length(7),   // exactly 7 rows
    Constraint::Length(9),   // exactly 9 rows
    Constraint::Min(8),      // at least 8, takes remaining space
])
.split(area);

// chunks[0], chunks[1], chunks[2] are Rect areas to draw into
```

**Ruby analogy:** Think of it like ERB templates but for terminal cells. Each frame is a full re-render.

### Widgets

```rust
use ratatui::widgets::{Block, Borders, Paragraph, List, ListItem};
use ratatui::style::{Style, Color, Modifier};

// A bordered block (used for every panel in kefctl)
let block = Block::default()
    .title(" My Panel ")
    .borders(Borders::ALL)
    .border_style(Style::default().fg(Color::Cyan));

// Text with styling
let text = Paragraph::new("Hello")
    .style(Style::default().fg(Color::White))
    .block(block);

frame.render_widget(text, area);

// A selectable list (used for sidebar, source panel)
let items = vec![ListItem::new("Item 1"), ListItem::new("Item 2")];
let list = List::new(items)
    .highlight_style(Style::default().add_modifier(Modifier::BOLD));

frame.render_stateful_widget(list, area, &mut list_state);
```

**In kefctl:** Each panel is a function in `ui/*.rs` that takes `(frame, app, area)` and renders widgets into the area. `theme.block(title, focused)` is a helper that handles focus styling.

### Styled text (Spans and Lines)

```rust
use ratatui::text::{Span, Line};

// A line with mixed styling (like HTML <span> tags)
let line = Line::from(vec![
    Span::styled("Label: ", Style::default().fg(Color::DarkGray)),
    Span::styled("Value", Style::default().fg(Color::White)),
]);
```

**In kefctl:** Status panel, footer, and help overlay all use `Span`/`Line` for mixed styling within a single line.

## 5. Crossterm (terminal input)

Handles keyboard events. Similar to Ruby's `io/console` but cross-platform.

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn handle_key(key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => quit(),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => quit(),
        KeyCode::Up | KeyCode::Char('k') => move_up(),
        KeyCode::Enter => select(),
        _ => {}  // ignore unknown keys
    }
}
```

**In kefctl:** All keyboard handling is in `app.rs` → `handle_key()`. It dispatches to panel-specific handlers based on `self.focus` and `self.panel`.

**Important:** Crossterm emits `Press`, `Repeat`, and `Release` events. kefctl filters for `KeyEventKind::Press` only in `event.rs` to avoid double-processing. If you handle raw crossterm events elsewhere, always check `key.kind == KeyEventKind::Press`.

## 6. Clap (CLI parsing)

Like Ruby's `OptionParser` but derived from struct definitions.

```rust
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    speaker: Option<String>,     // --speaker 192.168.50.17

    #[arg(long)]
    demo: bool,                  // --demo

    #[command(subcommand)]
    command: Option<Commands>,   // discover, status, source, volume
}
```

**In kefctl:** See `cli.rs`. Run `cargo run -- --help` to see the generated help.

### ValueEnum for typed arguments

When a CLI argument has a fixed set of choices (like source names), use `ValueEnum` instead of parsing strings manually:

```rust
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SourceArg {
    Usb,
    Wifi,
    Bluetooth,
}

// In the subcommand:
Source {
    #[arg(value_enum)]
    source: Option<SourceArg>,
}
```

Clap automatically validates input, generates help text with valid values, and enables shell tab completion. No manual `match` block needed.

**In kefctl:** `SourceArg` in `cli.rs` uses this pattern. The `cmd_set_source` function in `main.rs` converts `SourceArg` → `Source` with a simple match.

## 7. Error Handling (thiserror)

kefctl defines all errors in `error.rs` using the `thiserror` crate:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KefError {
    #[error("network error: {0}")]           // Display format
    Network(#[from] reqwest::Error),          // auto-converts from reqwest errors

    #[error("API error (status {status}): {message}")]
    Api { status: u16, message: String },     // structured variant with named fields

    #[error("type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: &'static str, got: String },

    #[error("discovery error: {0}")]
    Discovery(String),

    #[error("config error: {0}")]
    Config(#[from] toml::de::Error),          // auto-converts from toml errors
}
```

Key patterns:
- **`#[from]`** — auto-generates `From<T>` so the `?` operator works: `let resp = client.get(url).send().await?;` converts `reqwest::Error` → `KefError::Network` automatically
- **`#[error("...")]`** — generates the `Display` implementation (what the user sees)
- **Named struct variants** — for errors with multiple fields (like `Api { status, message }`)

**Adding a new error variant:** Add it to the enum, add the `#[error]` format string, and if it wraps another error type, add `#[from]`.

## 8. Architecture Patterns

### Event loop

The TUI runs an async event loop in `main.rs` → `run_tui_loop`. Three concurrent tasks feed events into a single `mpsc` channel:

```
Terminal Events (crossterm) ──┐
Speaker Poll (HTTP long-poll) ──┼──→ mpsc channel ──→ Event Handler ──→ App state
SIGUSR1 Listener ─────────────┘
```

The main loop receives events and dispatches:
- **`Event::Key`** → `app.handle_key()` → returns `Option<Action>` → `dispatch_action()`
- **`Event::Tick`** → `app.tick()` (advance progress bar, dismiss notifications)
- **`Event::SpeakerUpdate`** → replace `app.speaker` with fresh state from poll
- **`Event::SpeakerError`** → show notification, mark disconnected
- **`Event::ThemeChanged`** → reload theme from Omarchy colors.toml

### Optimistic updates with error feedback

When the user presses a key (e.g., volume up):

1. `app.handle_key()` updates `SpeakerState` **immediately** (instant UI feedback)
2. Returns `Some(Action::SetVolume(51))` to the main loop
3. `dispatch_action()` spawns an async HTTP request via `tokio::spawn`
4. If the API call **succeeds**: the speaker poll loop confirms the value on next poll
5. If the API call **fails**: error is sent back via `tx.send(Event::SpeakerError(...))`, shown as a TUI notification

This is not fire-and-forget — errors bubble back to the user.

### Speaker resolution chain

The speaker IP is resolved in order (first match wins):

1. `--speaker <ip>` CLI flag
2. `speaker.ip` in `~/.config/kefctl/config.toml`
3. Cached IP from last successful connection (`~/.local/state/kefctl/last_speaker`) — quick probe, falls back if unreachable
4. mDNS discovery (`_kef-info._tcp.local.`) — uses first speaker found

After a successful connection, the IP is saved to the cache file so subsequent launches skip the 5-second mDNS discovery.

### Visibility: `pub(crate)`

kefctl is a single binary — nothing is used outside the crate. All public items use `pub(crate)` instead of `pub` to prevent accidental API surface. When adding new structs, enums, or functions, use `pub(crate)` unless they're private to the module.

### `#[must_use]` on action-producing functions

`handle_key()` returns `Option<Action>` that must be dispatched to the API. The `#[must_use]` attribute makes the compiler warn if you accidentally drop the return value:

```rust
#[must_use]
pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Option<Action> { ... }
```

## 9. Theme System

All colors flow through `app.theme` (a `Theme` struct with 13 color fields). The `theme.block(title, focused)` helper builds styled borders. `theme.info_row(label, value)` and `theme.section_block(title)` provide consistent styling across panels.

### Omarchy integration

If [Omarchy](https://github.com/basecamp/omarchy) is installed, colors are read from `~/.config/omarchy/current/theme/colors.toml`. Missing colors fall back to defaults — the theme loads partially rather than failing entirely.

### Live reload via SIGUSR1

The event loop spawns a SIGUSR1 listener. When the signal fires, it sends `Event::ThemeChanged`, which triggers `Theme::load()`. To reload:

```sh
pkill -USR1 kefctl
```

Omarchy can trigger this automatically when themes change via a hook in `~/.config/omarchy/hooks/theme-set.d/kefctl`.

## 10. Logging and Debugging

### Log file

The TUI owns stdout, so all logging goes to `~/.local/state/kefctl/kefctl.log`. The `tracing` crate is used instead of `println!`.

### KEFCTL_LOG environment variable

Control log verbosity at runtime without recompiling:

```sh
# Default: info level for kefctl
cargo run -- --demo

# Debug logging for everything
KEFCTL_LOG=debug cargo run -- --demo

# Trace API calls only
KEFCTL_LOG=kefctl::kef_api=trace cargo run -- --speaker 192.168.50.17

# Silence everything except warnings
KEFCTL_LOG=warn cargo run -- --demo
```

The filter syntax is from `tracing_subscriber::EnvFilter` — `module=level` pairs separated by commas.

### #[tracing::instrument]

Key async API methods are annotated with `#[tracing::instrument]` which automatically creates a span with the function name and arguments:

```rust
#[tracing::instrument(skip(self), fields(path))]
pub async fn get_data(&self, path: &str) -> Result<GetDataResponse, KefError> { ... }
```

`skip(self)` avoids printing the entire `KefClient` struct. The span survives across `.await` points, so all log entries within the function are correlated.

**When adding new API methods**, add `#[tracing::instrument(skip(self))]` for automatic structured logging.

### Debugging against a real speaker

```sh
# Watch logs in real time
tail -f ~/.local/state/kefctl/kefctl.log

# Test API endpoints directly
curl -s 'http://192.168.50.17/api/getData?path=settings:/deviceName&roles=value' | python3 -m json.tool

# Use kefw2 Go CLI for quick speaker checks
kefw2 -s 192.168.50.17 info
```

## 11. Testing

kefctl has 99 tests across several categories.

### Test organization

Tests live inside each module in `#[cfg(test)] mod tests { ... }` blocks. This is standard Rust — tests are co-located with the code they test.

### Unit tests (state, types, config, errors)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App { App::new_demo() }  // helper: creates a demo app

    fn key(code: KeyCode) -> KeyEvent {  // helper: creates a key event
        KeyEvent { code, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE }
    }

    #[test]
    fn volume_up_clamped_at_max() {
        let mut a = app();
        a.speaker.volume = a.speaker.max_volume;
        a.handle_key(key(KeyCode::Char('+')));
        assert_eq!(a.speaker.volume, a.speaker.max_volume);
    }
}
```

**In kefctl:** `app.rs` has ~30 tests for keyboard handling and state transitions. `types.rs` has ~20 tests for serde roundtrips. `config.rs` has 9 tests for parsing. `error.rs` has 4 tests for Display formatting.

### UI rendering tests (TestBackend)

Ratatui's `TestBackend` renders to an in-memory buffer instead of a real terminal:

```rust
fn render_app(app: &mut App, width: u16, height: u16) -> ratatui::buffer::Buffer {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| super::draw(frame, app)).unwrap();
    terminal.backend().buffer().clone()
}

#[test]
fn status_panel_renders() {
    let mut app = App::new_demo();
    let buf = render_app(&mut app, 80, 24);
    let text = buffer_text(&buf);
    assert!(text.contains("Living Room LSX II"));
}
```

### Snapshot tests (insta)

For visual regression testing, kefctl uses the `insta` crate. Snapshot tests capture the full rendered output and compare against a saved `.snap` file:

```rust
#[test]
fn snapshot_status_panel() {
    let mut app = App::new_demo();
    app.select_panel(Panel::Status);
    let buf = render_app(&mut app, 80, 24);
    insta::assert_snapshot!(buffer_text(&buf));
}
```

Snapshots are stored in `src/ui/snapshots/`. When a UI change causes a snapshot mismatch:

```sh
cargo test                     # fails with diff
cargo insta review             # interactive review: accept or reject each change
cargo insta accept             # accept all pending changes
```

**When you change UI rendering**, run `cargo test` and review the snapshot diffs. If the change is intentional, accept the new snapshots.

### CI

GitHub Actions (`.github/workflows/ci.yml`) runs on every push and PR:

1. `cargo clippy --all-targets -- -D warnings` — lint with warnings as errors
2. `cargo test` — all 99 tests including snapshot tests
3. `cargo doc --no-deps` — verify documentation builds cleanly
4. `cargo build --release` — verify release build with LTO

## 12. Practical Workflow

### Making a change

```sh
# 1. Edit code
# 2. Check it compiles (fast feedback)
cargo check

# 3. Run in demo mode to see UI changes
cargo run -- --demo

# 4. Run against real speaker
cargo run -- --speaker 192.168.50.17

# 5. Lint and test before committing
cargo clippy
cargo test
```

### Common compiler errors and fixes

| Error | Fix |
|-------|-----|
| "value moved" | Add `&` to borrow, or `.clone()` to copy |
| "doesn't live long enough" | Store the value in a variable before borrowing |
| "mismatched types" | Check if you need `.to_string()`, `&`, or `*` |
| "non-exhaustive match" | Add the missing enum variant to your `match` |
| "unused variable" | Prefix with `_` like `_unused` or remove it |
| "trait not imported" | Add `use` for the trait (compiler suggests it) |
| "unused must_use" | Handle the return value (e.g., `let _ = ...` if intentional) |

### Cargo commands cheat sheet

| Command | Ruby equivalent |
|---------|----------------|
| `cargo check` | `ruby -c` (syntax check, but also types) |
| `cargo build` | — (Ruby is interpreted) |
| `cargo run` | `ruby main.rb` |
| `cargo test` | `rake test` / `rspec` |
| `cargo clippy` | `rubocop` |
| `cargo doc --open` | `yard doc` |
| `cargo add serde` | `bundle add serde` (adds to Cargo.toml) |
| `cargo insta review` | — (review snapshot test changes) |

## 13. How-To Guides

### Adding a new panel

1. Create `src/ui/mypanel.rs` with `pub fn draw(frame, app, area)`
2. Add variant to `Panel` enum in `app.rs`, update `ALL`, `label()`, `index()`
3. Wire it in `ui/mod.rs` match and `app.rs` keyboard handling
4. Use `theme.block(title, focused)` for borders, `app.theme.*` for colors
5. Add a snapshot test in `ui/mod.rs` tests
6. Run `cargo insta accept` to save the initial snapshot

### Adding a new API field

1. Add the field to `SpeakerState` in `app.rs`
2. Test the API endpoint: `curl -s 'http://<ip>/api/getData?path=<path>&roles=value'`
3. Add an `ApiValue` variant in `types.rs` if it's a new type (both `#[serde(tag)]` and `#[serde(rename)]` are needed — follow existing patterns)
4. Fetch it in `fetch_full_state()` in `kef_api/mod.rs`
5. Display it in the relevant `ui/*.rs` panel

### Adding a new CLI subcommand

1. Add the variant to `Commands` enum in `cli.rs`
2. Add a handler function in `main.rs`
3. Wire it in the `match cli.command` block in `main()`
4. For enum arguments, use `#[derive(ValueEnum)]` + `#[arg(value_enum)]`

### Adding a new error variant

1. Add the variant to `KefError` in `error.rs`
2. Add `#[error("...")]` format string
3. If wrapping another error type, add `#[from]` for automatic `?` conversion
4. Add a Display test in `error.rs` tests

## 14. Where to Go Next

- [The Rust Book](https://doc.rust-lang.org/book/) — chapters 1-10 cover everything in kefctl
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) — learn by reading code
- [Ratatui docs](https://docs.rs/ratatui/latest/ratatui/) — widget gallery and examples
- [Ratatui book](https://ratatui.rs/) — tutorials and recipes
- [Tokio tutorial](https://tokio.rs/tokio/tutorial) — async runtime used in kefctl
- [Serde guide](https://serde.rs/) — serialization framework
- [thiserror docs](https://docs.rs/thiserror/latest/thiserror/) — error derive macro
- [insta docs](https://insta.rs/) — snapshot testing
- [tracing docs](https://docs.rs/tracing/latest/tracing/) — structured logging
