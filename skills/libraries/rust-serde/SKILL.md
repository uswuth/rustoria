---
name: rust-serde
description: Use when serializing/deserializing Rust data.
---

# Serde — Serialization Framework for Rust

## Overview

Serde is a framework for serializing and deserializing Rust data structures efficiently and generically. It supports
JSON, TOML, MessagePack, CBOR, YAML, and many more formats.

## Installation

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }  # 1.0.2xx series
serde_json = "1"      # JSON
toml = "1"           # TOML (1.x line; 0.8 still works but is the previous major)
rmp-serde = "1"       # MessagePack
ciborium = "0.2"      # CBOR
# chrono = { version = "0.4", features = ["serde"] }  # if you serialize DateTime
```

> **YAML warning**: neither YAML serde adapter is a safe long-term bet. `serde_yaml` is archived
> (`0.9.34+deprecated`) and `serde_yml` is itself now marked deprecated/unmaintained on crates.io (its releases
> forward to the `noyalib` crate). For new projects prefer JSON or TOML; if you must have YAML, evaluate
> `noyalib` directly and pin what you verify.

## Core Concepts

### 1. Basic Derive

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let point = Point { x: 1, y: 2 };

    // Serialize to JSON
    let json = serde_json::to_string(&point).unwrap();
    println!("Serialized: {}", json); // {"x":1,"y":2}

    // Deserialize from JSON
    let deserialized: Point = serde_json::from_str(&json).unwrap();
    println!("Deserialized: {:?}", deserialized);
}
```

### 2. Field Attributes

```rust
use chrono::{DateTime, Utc};   // chrono = { version = "0.4", features = ["serde"] }
use serde::{Deserialize, Serialize};
use serde_json::Value;         // serde_json = "1"
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct User {
    #[serde(rename = "user_id")]
    id: u64,

    #[serde(default)]
    name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,

    // Deserializes as String::default(); the field type must implement Default
    #[serde(skip)]
    password: String,

    #[serde(rename(serialize = "createdAt", deserialize = "created_at"))]
    created_at: DateTime<Utc>,

    #[serde(default)]
    metadata: HashMap<String, Value>,
}
```

### 3. Container Attributes

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct User {
    first_name: String,
    last_name: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Message {
    Text { content: String },
    Image { url: String, width: u32 },
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Value {
    Int(i64),
    Float(f64),
    Str(String),
}
```

### 4. Custom Serialization

```rust
use serde::{Deserialize, Deserializer, Serialize, Serializer};

struct Celsius(f64);

impl Serialize for Celsius {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_f64(self.0)
    }
}

impl<'de> Deserialize<'de> for Celsius {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let value = f64::deserialize(deserializer)?;
        Ok(Celsius(value))
    }
}
```

### 5. Custom Field Serialization

```rust
use serde::{Deserialize, Serialize};
use std::time::Duration;

mod milliseconds {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_u64(duration.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where D: Deserializer<'de> {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

#[derive(Serialize, Deserialize)]
struct Event {
    #[serde(with = "milliseconds")]
    timestamp: Duration,
}
```

### 6. Formats

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let point = Point { x: 1, y: 2 };

    // JSON (serde_json)
    let json = serde_json::to_string(&point)?;
    let json_pretty = serde_json::to_string_pretty(&point)?;
    let back: Point = serde_json::from_str(&json)?;
    // also: from_reader(reader), from_slice(&bytes)
    assert_eq!(back, point);
    println!("{json_pretty}");

    // TOML (toml)
    let toml_str = toml::to_string(&point)?;
    let back: Point = toml::from_str(&toml_str)?;
    assert_eq!(back, point);

    // YAML - both serde_yaml and serde_yml are deprecated on crates.io (see YAML warning
    // above); prefer JSON/TOML. Shown for reading legacy files only.
    let yaml = serde_yml::to_string(&point)?;
    let back: Point = serde_yml::from_str(&yaml)?;
    assert_eq!(back, point);

    // MessagePack (rmp-serde)
    let bytes = rmp_serde::to_vec(&point)?;
    let back: Point = rmp_serde::from_slice(&bytes)?;
    assert_eq!(back, point);

    Ok(())
}
```

### 7. Enums

Four representations — pick **one** per enum (these are alternatives, not meant to coexist in one file):

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Externally tagged (default)
#[derive(Serialize, Deserialize)]
enum Message {
    Request { id: u64, method: String, params: Vec<Value> },
    Response { id: u64, result: Value },
}
```

```rust
// Internally tagged
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Message {
    Request { id: u64, method: String },
    Response { id: u64, result: Value },
}
```

```rust
// Adjacently tagged
#[derive(Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum Message {
    Request { id: u64, method: String },
    Response { id: u64, result: Value },
}
```

```rust
// Untagged — tried in order; first variant that matches wins
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Value {
    Int(i64),
    Float(f64),
    Str(String),
}
```

### 8. Generic Types

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    data: T,
    status: String,
}

#[derive(Serialize, Deserialize)]
struct Paginated<T> {
    items: Vec<T>,
    total: u64,
    page: u64,
}
```

### 9. Remote Derive (for external types)

`with` and `remote` solve different problems:

- `with = "module"` — use a module's `serialize`/`deserialize` functions for one field.
- `remote = "path"` — derive `Serialize`/`Deserialize` for a type from another crate you don't own, by mirroring its
  definition. The generated `...Def` type is then referenced from fields with `with`.

```rust
use serde::{Deserialize, Serialize};

// Imagine `External` lives in another crate and has no serde impls.
mod external_crate {
    pub struct External {
        pub a: i32,
        pub b: String,
    }
}

use external_crate::External;

// Mirror the external type's fields, pointed at the real type:
#[derive(Serialize, Deserialize)]
#[serde(remote = "External")]
struct ExternalDef {
    a: i32,
    b: String,
}

// Then apply it to fields with `with`:
#[derive(Serialize, Deserialize)]
struct Wrapper {
    name: String,
    #[serde(with = "ExternalDef")]
    external: External,
}

fn main() -> serde_json::Result<()> {
    let w = Wrapper {
        name: "demo".into(),
        external: External { a: 1, b: "x".into() },
    };
    let json = serde_json::to_string(&w)?;
    println!("{json}");
    Ok(())
}
```

### 10. Best Practices

1. **Use `#[serde(default)]`** for backward-compatible deserialization
2. **Use `skip_serializing_if`** to omit empty fields
3. **Use `rename_all`** for consistent naming conventions
4. **Use `with` module** for custom field serialization
5. **Use `try_from`** for validation during deserialization
6. **Use `flatten`** for embedding structs
7. **Use `borrow`** for zero-copy deserialization: `&'de str`
8. **Use `serde_json::Value`** for dynamic JSON

## When to Use Serde

- API request/response serialization
- Configuration file parsing
- Database row mapping
- Message queue serialization
- Any data format conversion
