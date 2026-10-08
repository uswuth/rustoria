# Type Design Patterns in Rust

## Overview

Rust's type system is expressive enough to encode domain invariants at the compile time. The goal: **make invalid states
unrepresentable**. If your code compiles, entire classes of bugs are impossible.

---

## Newtype Pattern

Wrap a primitive to give it semantic meaning and prevent mixing up values.

```rust
// Bad: easy to swap user_id and order_id
fn process(user_id: u64, order_id: u64) { /* ... */ }

// Good: distinct types prevent mix-ups
struct UserId(u64);
struct OrderId(u64);

fn process(user_id: UserId, order_id: OrderId) { /* ... */ }
```

### When to use newtypes

- **Domain primitives** — `Meters`, `Seconds`, `Email`, `Url`.
- **Type safety** — prevent mixing up IDs, units, or currencies.
- **Encapsulation** — hide internal representation, expose only safe operations.

### When NOT to use newtypes

- When the wrapper adds no semantic value (e.g., `struct Wrapper(String)` with no extra logic).
- When you need maximum performance and can't afford the (usually zero-cost) abstraction.

### Implementing traits on newtypes

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Port(u16);

impl Port {
    fn new(port: u16) -> Option<Self> {
        if port == 0 { None } else { Some(Self(port)) }
    }
}

// Prefer an explicit accessor or AsRef over Deref for newtypes:
impl Port {
    fn get(&self) -> u16 { self.0 }
}

impl AsRef<u16> for Port {
    fn as_ref(&self) -> &u16 { &self.0 }
}
```

**On `Deref`:** std's guidance is that `Deref` is for *smart pointers* — types
that own or point at another value (`Box`, `Vec`, `String`) — not for making
newtypes transparently behave like their inner type. Implementing `Deref` on
`Port` invites implicit deref coercion everywhere and raises the expectation of
a matching `DerefMut`, which would let callers write `*port = 0` and bypass the
validation in `Port::new`.

---

## Enums for State Machines

Model states as enum variants. Transitions are functions that consume one state and produce another.

```rust
enum Connection {
    Disconnected { reason: Option<DisconnectReason> },
    Connecting { attempts: u8 },
    Connected { stream: TcpStream },
}

impl Connection {
    /// One connection step: returns the next state.
    fn connect(self) -> Self {
        match self {
            Self::Disconnected { .. } => Self::Connecting { attempts: 1 },
            Self::Connecting { attempts } => match try_handshake() {
                Ok(stream) => Self::Connected { stream }, // success
                Err(_) if attempts < 3 => Self::Connecting {
                    attempts: attempts + 1,
                },
                Err(_) => Self::Disconnected {
                    reason: Some(DisconnectReason::Timeout),
                },
            },
            connected @ Self::Connected { .. } => connected, // already connected: no-op
        }
    }
}
```

Every state is reachable: `Disconnected` starts an attempt, `Connecting` either
succeeds into `Connected` or retries up to 3 attempts before giving up with a
recorded reason.

### Benefits

- **Exhaustive matching** — the compiler forces you to handle every state.
- **Invalid transitions are unrepresentable** — you can't be `Connected` without a `stream`.
- **Self-documenting** — the type system documents the state machine.

### When NOT to use enum state machines

- When states have many shared fields — consider an enum with a common struct, or the type-state pattern.
- When transitions are dynamic/data-driven — a runtime state machine (e.g., using a `HashMap` of transitions) may be
  more appropriate.

---

## Making Invalid States Unrepresentable

The core principle: **use types to rule out bad data at compile time**.

### Example: Non-empty string

```rust
// Bad: empty string is a valid String
fn send_email(to: &str) { /* ... */ }

// Good: NonEmptyStr guarantees at least one character
struct NonEmptyStr(String);

impl NonEmptyStr {
    fn new(s: String) -> Result<Self, EmptyError> {
        if s.is_empty() { Err(EmptyError) } else { Ok(Self(s)) }
    }
    fn as_str(&self) -> &str { &self.0 }
}
```

### Example: Bounded integer

```rust
struct Percentage(u8);

impl Percentage {
    fn new(value: u8) -> Option<Self> {
        if value <= 100 { Some(Self(value)) } else { None }
    }
    fn value(&self) -> u8 { self.0 }
}
```

The pattern is identical for other validated primitives — a validated `Email` is
`NonEmptyStr` plus a format check, and the type-safe IDs in the Newtype section
above are the zero-validation case. Two ingredients always: a private field and a
constructor that returns `Result`/`Option`.

### serde caveat: `derive(Deserialize)` bypasses validation

Deriving `Deserialize` on a validated newtype constructs the inner value
directly — no `new()`, no checks. Implement `Deserialize` manually, or route
through `try_from` so deserialization reuses the validating constructor:

```rust
#[derive(serde::Deserialize)]
#[serde(try_from = "String")]
struct NonEmptyStr(String);

impl TryFrom<String> for NonEmptyStr {
    type Error = EmptyError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}
```

---

## Type-State Pattern

Encode state in the type parameter. The compiler tracks state transitions at compile time.

```rust
struct Unauthenticated;
struct Authenticated { token: String }

struct ApiClient<State> {
    base_url: String,
    state: State,
}

impl ApiClient<Unauthenticated> {
    fn new(base_url: String) -> Self {
        Self { base_url, state: Unauthenticated }
    }

    fn authenticate(self, token: String) -> ApiClient<Authenticated> {
        ApiClient {
            base_url: self.base_url,
            state: Authenticated { token },
        }
    }
}

impl ApiClient<Authenticated> {
    fn get(&self, path: &str) -> Response {
        // can only call this when authenticated
        http_get(&self.base_url, path, &self.state.token)
    }
}

// Usage:
let client = ApiClient::new("https://api.example.com".into());
// client.get("/users"); // ERROR: `get` only exists on ApiClient<Authenticated>;
                         // this value is ApiClient<Unauthenticated>
let client = client.authenticate("secret-token".into());
client.get("/users"); // compiles — the type proves authentication happened
```

### When to use type-state

- **Builder patterns** — enforce that required fields are set before building.
- **State machines** — compile-time guarantee of valid transitions.
- **Resource management** — e.g., a file handle that must be opened before reading.

### When NOT to use type-state

- When states are determined at runtime (e.g., user input).
- When the complexity outweighs the benefit — a runtime check with a clear error may be simpler.
- When you need to store the value in a collection — different type-states are different types.

---

## PhantomData

Use `PhantomData` to mark ownership or lifetime relationships without adding a field.

```rust
use std::marker::PhantomData;

struct Id<'a, T> {
    value: u64,
    _marker: PhantomData<&'a T>,
}

// Now Id<'a, User> and Id<'a, Product> are distinct types
// and carry lifetime information
```

### Common uses

- **Lifetime markers** — tie a struct's lifetime to a borrow it doesn't directly hold.
- **Type markers** — distinguish `Id<User>` from `Id<Product>`.
- **Drop check** — `PhantomData<T>` affects drop order and variance.

---

## Zero-Sized Types (ZSTs)

Types with no runtime cost, used purely for type-level programming. Here
`Kilometers` and `Miles` are ZST *markers* — the conversion factor lives on a
`Unit` trait as an associated const, and `Distance<U>` carries the value:

```rust
use std::marker::PhantomData;

struct Kilometers;
struct Miles;

trait Unit {
    /// How many meters one unit of this type represents.
    const METERS: f64;
}

impl Unit for Kilometers {
    const METERS: f64 = 1000.0;
}

impl Unit for Miles {
    const METERS: f64 = 1609.344;
}

struct Distance<U> {
    value: f64,
    _unit: PhantomData<U>,
}

impl<U: Unit> Distance<U> {
    fn new(value: f64) -> Self {
        Self { value, _unit: PhantomData }
    }

    fn to_meters(&self) -> f64 {
        self.value * U::METERS
    }
}

// let d = Distance::<Kilometers>::new(5.0);
// assert_eq!(d.to_meters(), 5000.0);
// Distance<Kilometers> and Distance<Miles> are incompatible types — mixing
// units is a compile error, at zero runtime cost (PhantomData is a ZST).
```

---

## Summary

| Pattern | Use When | Benefit |
| --------- | ---------- | --------- |
| Newtype | Distinct semantics for same underlying type | Prevents mix-ups |
| Enum state machine | Finite states with transitions | Exhaustive matching |
| Validated newtypes | Input must satisfy invariants | Compile-time guarantees |
| Type-state | Compile-time state tracking | Zero runtime cost |
| PhantomData | Lifetime/type markers without fields | Type safety |
| ZSTs | Type-level programming | Zero runtime cost |
