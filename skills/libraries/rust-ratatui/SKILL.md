---
name: rust-ratatui
description: Use when building terminal UIs with Ratatui.
---

# Rust Ratatui — Terminal User Interfaces

## Overview

Ratatui is a Rust crate for building terminal user interfaces (TUIs). It provides a simple, flexible way to create
text-based UIs for CLIs, dashboards, and interactive console apps. It uses `crossterm` for terminal control.

## Installation

```toml
[dependencies]
ratatui = "0.30"
crossterm = "0.29"   # ratatui 0.30.x pairs with crossterm 0.29
color-eyre = "0.6"  # Optional: better error handling
```

> **Crate layout**: since 0.30, ratatui's core types live in the separate `ratatui-core` crate, and each backend
> (crossterm, termion, termwiz) is its own crate. The umbrella `ratatui` crate re-exports the crossterm backend, but the
> exact re-export path can shift between releases — verify against docs.rs for the exact version you pin.

## Core Concepts

### 1. Application Structure

Every Ratatui app follows this pattern:

```rust
use color_eyre::Result;
use crossterm::event::{self, Event};
use ratatui::{DefaultTerminal, Frame};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        terminal.draw(render)?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    frame.render_widget("hello world", frame.area());
}
```

### 2. The Render Loop

```rust
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};

// `App` is defined in §3.
fn run(mut terminal: DefaultTerminal, app: &mut App) -> Result<()> {
    loop {
        // 1. Draw the UI
        terminal.draw(|frame| render(frame, app))?;

        // 2. Handle events
        match event::read()? {
            // crossterm reports Press, Repeat, and Release (Release/Repeat
            // notably on Windows). Filtering for Press means one keystroke
            // is handled once — not again on release or auto-repeat.
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => break Ok(()),
                KeyCode::Left => app.decrement(),
                KeyCode::Right => app.increment(),
                _ => {}
            },
            _ => {}
        }
    }
}

fn render(frame: &mut Frame, app: &App) {
    frame.render_widget(format!("counter: {}", app.counter), frame.area());
}
```

### 3. App State

```rust
struct App {
    counter: i32,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self { counter: 0, should_quit: false }
    }

    fn increment(&mut self) {
        self.counter += 1;
    }

    fn decrement(&mut self) {
        self.counter -= 1;
    }

    fn quit(&mut self) {
        self.should_quit = true;
    }
}
```

### 4. Widgets

```rust
use ratatui::{
    widgets::{Block, Borders, Paragraph},
    Frame,
};

fn render(frame: &mut Frame, app: &App) {
    let text = Paragraph::new(format!("Counter: {}", app.counter))
        .block(Block::default().title("Counter App").borders(Borders::ALL));
    frame.render_widget(text, frame.area());
}
```

### 5. Layout

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(1),     // Content
            Constraint::Length(3),  // Footer
        ])
        .split(frame.area());

    // Header
    let header = Paragraph::new("My App")
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Content
    let content = Paragraph::new(format!("Counter: {}", app.counter))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(content, chunks[1]);

    // Footer
    let footer = Paragraph::new("Press q to quit")
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);
}
```

### 6. Styling

```rust
// Fragment — widget construction, to be rendered inside a render function.
use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
};

let styled = Paragraph::new("Hello")
    .style(Style::default()
        .fg(Color::Cyan)
        .bg(Color::Black)
        .add_modifier(Modifier::BOLD));

// Block with styled title
let block = Block::default()
    .title(Span::styled(
        "Title",
        Style::default().fg(Color::White).bg(Color::Blue),
    ))
    .borders(Borders::ALL)
    .border_style(Style::default().fg(Color::Yellow));
```

### 7. List Widget

```rust
// Fragment — `frame` and `area` come from the enclosing render function.
use ratatui::{
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

let items = vec![
    ListItem::new("Item 1"),
    ListItem::new("Item 2"),
    ListItem::new("Item 3"),
];

let list = List::new(items)
    .block(Block::default().title("Items").borders(Borders::ALL))
    .highlight_style(Style::default().add_modifier(Modifier::BOLD))
    .highlight_symbol("> ");

// With selection state
let mut state = ListState::default();
state.select(Some(0));
frame.render_stateful_widget(list, area, &mut state);
```

### 8. Table Widget

```rust
// Fragment — construct inside a render function, then `frame.render_widget(table, area)`.
use ratatui::{
    layout::Constraint,
    style::{Color, Style},
    widgets::{Block, Borders, Row, Table},
};

let header = Row::new(vec!["Name", "Age", "City"])
    .style(Style::default().fg(Color::Yellow))
    .height(1);

let rows = vec![
    Row::new(vec!["Alice", "30", "NYC"]),
    Row::new(vec!["Bob", "25", "LA"]),
];

let table = Table::new(rows, [Constraint::Length(10), Constraint::Length(5), Constraint::Length(10)])
    .header(header)
    .block(Block::default().title("Users").borders(Borders::ALL));
```

### 9. Gauge Widget

```rust
// Fragment — construct inside a render function, then `frame.render_widget(gauge, area)`.
use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Gauge},
};

let gauge = Gauge::default()
    .block(Block::default().title("Progress").borders(Borders::ALL))
    .gauge_style(Style::default().fg(Color::Cyan))
    .ratio(0.5);  // 50%
```

### 10. Event Handling

```rust
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

// Returns Ok(true) when the app should quit. `App` is defined in §3.
fn handle_events(app: &mut App) -> Result<bool> {
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
                KeyCode::Left => app.decrement(),
                KeyCode::Right => app.increment(),
                _ => {}
            }
        }
        Event::Mouse(_) => {}
        Event::Resize(_, _) => {}
        _ => {}
    }
    Ok(false)
}
```

### 11. Project Structure

For larger apps, organize into modules:

```text
src/
├── main.rs      # Entry point, terminal init/restore
├── app.rs       # App state and logic
├── ui.rs        # Render functions
├── events.rs    # Event handling
└── widgets.rs   # Custom widgets
```

```rust
// main.rs
mod app;
mod events;
mod ui;

use color_eyre::Result;
use ratatui::DefaultTerminal;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut app = app::App::new();
    loop {
        terminal.draw(|frame| ui::render(frame, &app))?;
        if events::handle_events(&mut app)? {
            break Ok(());
        }
    }
}
```

### 12. Templates

Use official templates to get started quickly:

```bash
cargo install --locked cargo-generate
cargo generate ratatui/templates
```

Available templates:

- `hello-world` — Minimal app
- `counter` — Counter with state
- `json-editor` — JSON editor TUI

## Best Practices

1. **Use `ratatui::init()` and `ratatui::restore()`** — handles terminal setup/teardown
2. **Separate state from rendering** — `app.rs` for state, `ui.rs` for rendering
3. **Use `color_eyre` for error handling** — better backtraces in terminal apps
4. **Filter `KeyEventKind::Press`** — crossterm emits Press/Repeat/Release (Release/Repeat notably on Windows);
   without the filter one keystroke gets handled multiple times
5. **Use `Layout` for responsive UIs** — don't hardcode positions
6. **Use `Constraint` for sizing** — `Length`, `Min`, `Max`, `Percentage`, `Ratio`
7. **Keep widgets pure** — render functions should not mutate state
8. **Use `render_stateful_widget` for selections** — ListState, TableState
9. **Clear before drawing popups** — use `Clear` widget
10. **Test app logic separately** — state logic is testable without terminal

## When to Use

- Terminal dashboards
- CLI tools with interactive UIs
- System monitors
- Text editors
- Any application that needs a rich terminal interface
