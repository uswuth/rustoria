---
name: rust-clap
description: Use when building CLI tools with clap in Rust.
---

# Rust Clap — Command Line Argument Parser

## Overview

Clap is a full-featured, fast command-line argument parser for Rust. It supports declarative and procedural parsing,
subcommands, completions, and extensive customization.

## Installation

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }  # 4.6 as of late 2026
```

## Core Concepts

### 1. Derive API (Recommended)

```rust
use clap::{ArgAction, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mytool", version, about, long_about = None)]
struct Cli {
    /// Input file path
    #[arg(short, long)]
    input: String,

    /// Output file path
    #[arg(short, long)]
    output: Option<String>,

    /// Enable verbose output
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new item
    Add {
        /// Name of the item
        name: String,
        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// List all items
    List {
        /// Filter by status
        #[arg(short, long)]
        filter: Option<String>,
    },
    /// Remove an item
    Remove {
        /// ID of the item to remove
        id: u64,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add { name, description }) => {
            println!("Adding: {} {:?}", name, description);
        }
        Some(Commands::List { filter }) => {
            println!("Listing with filter: {:?}", filter);
        }
        Some(Commands::Remove { id }) => {
            println!("Removing: {}", id);
        }
        None => {
            println!("Input: {}, Output: {:?}, Verbose: {}",
                cli.input, cli.output, cli.verbose);
        }
    }
}
```

### 2. Builder API

```rust
use clap::{Arg, ArgAction, Command};

fn build_cli() -> Command {
    Command::new("mytool")
        .version("1.0")
        .about("A sample CLI tool")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("Input file path")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file path"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::Count)
                .help("Enable verbose output"),
        )
        .subcommand(
            Command::new("add")
                .about("Add a new item")
                .arg(
                    Arg::new("name")
                        .required(true)
                        .help("Name of the item"),
                ),
        )
}

fn main() {
    let matches = build_cli().get_matches();

    let input = matches.get_one::<String>("input").unwrap(); // required, so always present
    let output = matches.get_one::<String>("output");
    let verbose = matches.get_count("verbose");

    println!("Input: {}, Output: {:?}, Verbose: {}", input, output, verbose);
}
```

### 3. Argument Types

```rust
use clap::{ArgAction, Parser};

#[derive(Parser)]
struct Cli {
    /// Positional argument
    name: String,

    /// Optional positional
    count: Option<u32>,

    /// Flag (boolean)
    #[arg(long)]
    force: bool,

    /// Option with value
    #[arg(short, long)]
    output: Option<String>,

    /// Multiple values
    #[arg(short, long, num_args = 1..)]
    files: Vec<String>,

    /// Count occurrences
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,

    /// Set if present
    #[arg(long, action = ArgAction::SetTrue)]
    dry_run: bool,

    /// Append values
    #[arg(long, action = ArgAction::Append)]
    define: Vec<String>,

    /// Possible values
    #[arg(long, value_enum)]
    format: Option<OutputFormat>,

    /// Value parser
    #[arg(short, long, value_parser = clap::value_parser!(u16))]
    port: Option<u16>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum OutputFormat {
    Json,
    Yaml,
    Toml,
}
```

### 4. Subcommands

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new resource
    Create {
        /// Name of the resource
        name: String,
        #[arg(short, long)]
        tags: Vec<String>,
    },
    /// Get a resource
    Get {
        /// ID of the resource
        id: u64,
        /// Output format
        #[arg(short, long, value_enum)]
        format: Option<OutputFormat>,
    },
    /// Delete a resource
    Delete {
        /// ID of the resource
        id: u64,
        /// Skip confirmation
        #[arg(long)]
        force: bool,
    },
}
```

(`OutputFormat` is the `ValueEnum` from §3.)

### 5. Value Parsers

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    /// Custom value parser
    #[arg(short, long, value_parser = parse_hex)]
    color: Option<u32>,

    /// Path with validation
    #[arg(short, long, value_parser = clap::builder::PathBufValueParser::new())]
    config: Option<PathBuf>,

    /// Number range
    #[arg(short, long, value_parser = clap::value_parser!(u32).range(1..=100))]
    threads: Option<u32>,
}

fn parse_hex(s: &str) -> Result<u32, String> {
    u32::from_str_radix(s.trim_start_matches("#"), 16)
        .map_err(|e| format!("Invalid hex color: {}", e))
}
```

### 6. Help and Version

```rust
use clap::Parser;

#[derive(Parser)]
#[command(
    name = "mytool",
    version,
    about = "A sample CLI tool",
    long_about = "A longer description of the tool",
    author = "Your Name",
    max_term_width = 100
)]
struct Cli {
    #[arg(help = "Input file", long_help = "Path to the input file to process")]
    input: String,
}
```

### 7. Error Handling

`Cli::parse()` prints clap's formatted error and exits on failure. Handle manually only when you need custom control —
and keep clap's formatting by calling `Error::exit()` (which prints the message and exits with the right code) instead
of a bare `unwrap()`:

```rust
use clap::{CommandFactory, FromArgMatches, Parser};

#[derive(Parser)]
#[command(name = "mytool", version, about)]
struct Cli {
    /// Input file path
    #[arg(short, long)]
    input: Option<String>,
}

fn main() {
    let matches = Cli::command().try_get_matches().unwrap_or_else(|e| e.exit());
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
    println!("input = {:?}", cli.input);
}
```

### 8. Shell Completions

```toml
[dependencies]
clap_complete = "4"
```

```rust
use clap::CommandFactory;
use clap_complete::{generate, Shell};

// Fragment: assumes the `Cli` struct from §7.
fn main() {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(Shell::Bash, &mut cmd, name, &mut std::io::stdout());
}
```

### 9. Configuration Files

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
toml = "1"
```

```rust
use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    config: Option<PathBuf>,

    #[arg(short, long)]
    output: Option<String>,
}

#[derive(Default, Deserialize)]
struct Config {
    output: Option<String>,
    verbose: bool,
}

// Result-returning main so `?` works
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Load config file if specified
    let config: Config = if let Some(path) = &cli.config {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content)?
    } else {
        Config::default()
    };

    // CLI args override config
    let output = cli.output.or(config.output);
    println!("output = {output:?}, verbose = {}", config.verbose);
    Ok(())
}
```

### 10. Testing

```toml
[dev-dependencies]
assert_cmd = "2"
predicates = "3"
```

```rust
use assert_cmd::Command;
use clap::CommandFactory;
use predicates::prelude::*;

// Compile-time check of the derive definition (uses the `Cli` from §1)
#[test]
fn verify_cli() {
    Cli::command().debug_assert();
}

#[test]
fn test_add_command() {
    let mut cmd = Command::cargo_bin("mytool").unwrap();
    cmd.args(["add", "test-item"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Adding: test-item"));
}

#[test]
fn test_missing_required() {
    let mut cmd = Command::cargo_bin("mytool").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}
```

## Best Practices

1. **Use derive API** for most cases — it's cleaner and more maintainable
2. **Use `value_enum`** for constrained choices — better than raw strings
3. **Use `value_parser`** for custom validation — fail early with clear errors
4. **Use `long_help`** for detailed explanations — keep `help` short
5. **Use `num_args`** for multiple values — clearer than manual parsing
6. **Use `ArgAction::Count`** for verbose flags — idiomatic
7. **Use `CommandFactory::debug_assert`** in tests — catches derive misconfigurations
8. **Use `clap_complete`** for shell completions — better UX
9. **Handle errors gracefully** — `try_get_matches` + `unwrap_or_else(|e| e.exit())` preserves clap's formatting
10. **Document with doc comments** — they become help text

## When to Use

- Command-line tools
- Developer utilities
- System administration tools
- Build tools
- Any terminal-based application
