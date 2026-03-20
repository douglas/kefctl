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

**In kefctl:** `fetch_full_state()` uses `tokio::try_join!` to fetch all settings in parallel. Action dispatch uses `tokio::spawn` for fire-and-forget API calls. The event loop in `event.rs` uses `tokio::select!` to wait on multiple event sources.

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

## 7. Practical Workflow

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

### Debugging

```sh
# Check logs (TUI takes over stdout, so logs go to file)
tail -f ~/.local/state/kefctl/kefctl.log

# Test API endpoints directly
curl -s 'http://192.168.50.17/api/getData?path=settings:/deviceName&roles=value' | python3 -m json.tool

# Use kefw2 Go CLI for quick speaker checks
kefw2 -s 192.168.50.17 info
kefw2 -s 192.168.50.17 status
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

## 8. Where to Go Next

- [The Rust Book](https://doc.rust-lang.org/book/) — chapters 1-10 cover everything in kefctl
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) — learn by reading code
- [Ratatui docs](https://docs.rs/ratatui/latest/ratatui/) — widget gallery and examples
- [Ratatui book](https://ratatui.rs/) — tutorials and recipes
- [Tokio tutorial](https://tokio.rs/tokio/tutorial) — async runtime used in kefctl
- [Serde guide](https://serde.rs/) — serialization framework
