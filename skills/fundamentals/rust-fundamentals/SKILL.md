---
name: rust-fundamentals
description: Use when learning or reviewing Rust language basics.
---

# rust-fundamentals — The Rust Language

Canonical native-Rust foundation. Reference-oriented: concise rules, small correct examples, decision guidance. For deep
dives see `references/ownership.md`, `references/borrowing.md`, `references/type-design.md`.

## When to use

- Learning or reviewing Rust from zero (variables through traits and generics).
- Teaching or explaining Rust to others (see Pedagogy below).
- Reviewing code for language-level correctness.
- Answering "which language feature fits this?" questions.

## Core rules

- **Ownership:** every value has exactly one owner; when the owner goes out of scope, the value is dropped. Moving a
  value transfers ownership; the original binding is invalid.
- **Borrowing:** `&T` shared borrows (any number) XOR `&mut T` exclusive borrow (one at a time) — never both live at
  once. Borrows must not outlive the referent.
- **Copy vs Clone:** `Copy` types (integers, bools, chars, references, tuples/arrays of Copy) duplicate implicitly on
  assignment. `Clone` is explicit (`.clone()`) and may be expensive. `Copy` requires all fields `Copy` and no `Drop`
  impl; `Box<T>` is never `Copy`.
- **Lifetimes:** describe how long references are valid. Elision covers most cases; annotate when the compiler cannot
  infer. `T: 'static` means no non-`'static` borrows inside (a `&'static str` satisfies it).

## Language tour

### Variables, mutability, constants, statics

```rust
let x = 5;              // immutable by default
let mut y = 5;          // mutable
const MAX: u32 = 100;   // compile-time constant, must be typed
static NAME: &str = "app"; // 'static lifetime, single location
```

- Prefer `let` bindings; use `mut` only when needed.
- `const` for compile-time values; `static` for global state (rare; `static mut` is unsafe and restricted in edition
  2024 — use atomics or `OnceLock` instead).
- Shadowing (`let x = ...; let x = ...`) is idiomatic for transformations.

### Functions, closures, expressions

```rust
fn add(x: i32, y: i32) -> i32 { x + y }   // last expression is the return value
let f = |x: i32| x * 2;                    // closure
let v = { let a = 1; a + 2 };              // block is an expression
```

- Functions: parameters typed, `->` return type, no semicolon on the returned expression.
- Closures: `|args| body`; capture by reference by default, `move` to take ownership.
- `if`/`match` are expressions — they return values. Statements end with `;`.

### Control flow

- `if`/`else if`/`else`, `match` (exhaustive — compiler enforces all arms), `if let`/`while let` for single-pattern binding.
- Loops: `loop` (with `break value`), `while`, `for x in iter` (idiomatic iteration; arrays are directly iterable —
  no `.iter()` needed on array literals).
- `?` propagates errors in functions returning `Option` or `Result`; `fn main() -> Result<(), E: Debug>` is stable.

### Structs, enums, pattern matching

```rust
struct User { name: String, age: u32 }          // named fields
struct Point(i32, i32);                        // tuple struct
struct Unit;                                   // unit struct (marker)
enum Shape { Circle(f64), Rect(f64, f64) }      // variants carry data
```

- Enums are the primary tool for modeling state; make invalid states unrepresentable (typestate pattern — see
  type-design reference).
- `match` on enums must be exhaustive; use `_` only when truly ignoring.
- Destructuring: `let` only takes irrefutable patterns (structs, tuples, single-variant enums) - refutable
  patterns like `Shape::Circle(r)` on a multi-variant enum need `if let Shape::Circle(r) = s { .. }` or `match`.

### Generics, traits, associated types

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T { /* ... */ }
trait Summary { fn summarize(&self) -> String; }
fn notify(item: &impl Summary) { /* static dispatch */ }
fn notify_dyn(item: &dyn Summary) { /* dynamic dispatch */ }
```

- Trait bounds: `T: Trait`, multiple with `+`, `where` clauses for complex bounds.
- `impl Trait` in argument position (static dispatch) vs `dyn Trait` (dynamic dispatch, object safety required —
  traits with generic methods or `Self` returns are not dyn-compatible).
- Associated types vs generics: associated type when each impl has one natural type; generic parameter when callers choose.
- `Self` refers to the implementing type; `Self: Sized` bound allows sized methods on dyn-unsafe traits.

### Lifetimes, ownership, borrowing, moves

```rust
let s = String::from("hi");
let s2 = s;                    // move — s is invalid now
let n = 5; let n2 = n;         // Copy — n still valid
let r = &s2;                   // borrow
```

- Return owned data from functions; return references only to data that outlives the call (parameters, `&self`, `'static`).
- E0515 (returning a reference to a local): return an owned value, or a genuine `static`, or (rarely) `Box::leak`.
- `&String` as a parameter is an anti-pattern — take `&str` (clippy `ptr_arg`).
- Raw pointers (`*const T`, `*mut T`) are for FFI/unsafe only; creating them is safe, dereferencing is not.

### References, slices, String, str

- `&T`/`&mut T` — references; `&[T]` — slice (borrowed view); `str` — UTF-8 string slice.
- `String` is owned UTF-8; `&str` is a borrowed view. Pass `&str`, return `String`.
- Slices: `&v[1..3]`, `&v[..]`, `&v[2..]`; panics on out-of-bounds or non-char-boundary.
- `String` ↔ `&str`: `&s` (deref coercion), `s.as_str()`, `String::from("x")`, `"x".to_string()`.

### Iterators

- Lazy: nothing runs until a consuming call. Chain `map`/`filter`/`flat_map`/`filter_map` without intermediate `collect()`.
- `for x in v` consumes `Vec`; `for x in &v` borrows; `for x in v.iter()` is explicit borrowing.
- Collect into `Vec`, `String`, `HashMap`, `Result<Vec<_>, E>` (fail-fast), `Option<Vec<_>>`.
- See `rust-std` for the full combinator map.

### Type aliases, newtypes, DSTs, dyn, impl Trait, Self, where

- `type Meters = f64;` — alias (no new type). `struct Meters(f64);` — newtype (new type, zero cost, enforces units).
- DSTs (`str`, `[T]`, `dyn Trait`) are unsized — always behind a pointer (`&str`, `Box<dyn Trait>`).
- `dyn Trait` = trait object (dynamic dispatch); `impl Trait` = opaque concrete type (static dispatch).
- `where T: Trait + Send` keeps signatures readable.

### Macros, attributes, derive, cfg

- `println!`, `vec!`, `format!` — declarative/stdlib macros; `macro_rules!` for simple metaprogramming; proc macros
  (`#[derive(...)]`) for code generation.
- Attributes: `#[derive(Debug, Clone, PartialEq)]`, `#[test]`, `#[cfg(test)]`, `#[allow(...)]`, `#[inline]`, `#[must_use]`.
- `#[cfg(feature = "x")]` / `cfg!` for conditional compilation; `#[cfg_attr]`.
- Derives: `Debug`, `Clone`, `Copy`, `PartialEq`/`Eq`, `PartialOrd`/`Ord`, `Hash`, `Default`,
  `Serialize`/`Deserialize` (serde).

## Advanced concepts

### Drop, Deref, DerefMut

- `Drop::drop` runs when a value goes out of scope. Drop order: locals in reverse declaration order; struct fields in
  declaration order; tuple/array elements in order; parameters after the body; closure captures in capture order.
- `ManuallyDrop<T>` suppresses drop (for unsafe transfers); `mem::forget` leaks (safe but usually wrong).
- `Deref`/`DerefMut` are for smart pointers (Box, Rc, Arc, Cow) — not for newtypes. Prefer `AsRef` or an explicit
  accessor for newtypes.

### Conversion traits

Implement `From` (not `Into` — it comes free). `TryFrom`/`TryInto` for fallible conversions. `AsRef` for
borrow-generic parameters. `Borrow` for hash-map key equivalence. `FromStr` for parsing. `Display` for user output,
`Debug` for diagnostics. See `rust-std` for the full table.

### Operator traits, indexing, formatting

- `std::ops`: `Add`, `Sub`, `Index`/`IndexMut` - operator and indexing traits; implement where the operation is
  meaningful for domain types. (`PartialEq`/`Eq`/`PartialOrd`/`Ord` live in `std::cmp`, not `std::ops`.)
- `{}` uses `Display`; `{:?}` uses `Debug`; `{:#?}` pretty-prints; `{:x?}` hex.
- `write!`/`writeln!` to any `fmt::Write` (String, files, buffers).

### Allocation and layout

- Stack: fast, fixed size, scoped. Heap: `Box`, `Vec`, `String` — flexible, owned, freed on drop.
- `Box<T>` for heap allocation, recursive types, large moves, trait objects.
- `repr(C)` for FFI/ABI layout; default `repr(Rust)` lets rustc reorder fields for padding (manual field ordering
  matters mainly for `repr(C)` and cache-line layout).
- `size_of::<T>()`, `align_of::<T>()` for layout questions.

### Common compiler errors

| Error | Meaning | Fix |
| ------- | --------- | ----- |
| E0384 | cannot assign twice to immutable | add `mut` or shadow |
| E0507 | cannot move out of borrowed content | clone, borrow, or restructure |
| E0515 | cannot return reference to local | return owned / `static` / `Box::leak` |
| E0597 | borrow does not live long enough | shorten borrow, extend data lifetime, or own the data |
| E0308 | mismatched types | check types, deref coercion, or convert |
| E0432 | unresolved import | check `use` path and module visibility |
| E0433 | failed to resolve: undeclared crate or module | add the dependency, fix the path, check module visibility |

## Pedagogy (teaching or explaining Rust)

- Use concrete analogies for ownership (a toy: one owner, lending = borrowing, giving = moving) — then show the
  compiler enforcing it.
- Show real compiler errors and fix them live; the compiler is the best teacher.
- Build incrementally: one concept per step; run `cargo run` often.
- Use `dbg!(&value)` to inspect values at runtime.
- Draw ownership/borrow diagrams for non-obvious cases.
- Relate new concepts to what the learner already knows, then show where Rust differs.

## House rules

- Native first: `std` before crates (see `rust-std`).
- No blocking in async; no casual `unwrap()` in production paths.
- Version-sensitive advice names its version.
- Examples in this repository use edition 2021.
