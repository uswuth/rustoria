# CLI Tool

A command-line todo list manager built with clap.

## Features

- **Clap argument parsing** - Derive-based CLI with subcommands
- **Persistent storage** - JSON file-based storage
- **Error handling** - Proper error propagation with anyhow
- **Testing** - Integration tests with assert_cmd and tempfile

## Project Structure

```text
src/
├── main.rs           # Entry point
├── cli.rs            # Clap argument parsing
├── commands/         # Command implementations
│   ├── mod.rs
│   ├── add.rs        # Add command
│   ├── list.rs       # List command
│   └── remove.rs     # Remove command
├── models.rs         # Data models
└── storage.rs        # Persistence layer
```

## Running

```bash
# Add a todo
cargo run -- add "Buy groceries" --priority high --tag shopping

# List all todos
cargo run -- list

# List with filters
cargo run -- list --status pending --limit 10

# Remove a todo
cargo run -- remove <id>

# Run tests
cargo test
```

## Commands

| Command | Description | Options |
| --------- | ------------- | --------- |
| `add` | Add a new todo | `--priority`, `--tag` |
| `list` | List todos | `--status`, `--limit` |
| `remove` | Remove a todo by ID | - |

## What It Demonstrates

- **Clap derive API** - Clean, declarative argument parsing
- **Subcommand organization** - Each command in its own module
- **Error handling** - anyhow for ergonomic error propagation
- **File I/O** - JSON serialization with serde
- **Testing** - Integration tests with assert_cmd and tempfile
- **Environment-based configuration** - Storage path via environment variable
