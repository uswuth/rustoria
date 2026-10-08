# Ownership in Rust

## Overview

Ownership is Rust's central memory-management mechanism. The compiler enforces two ownership rules at compile time:

1. Every value has exactly one **owner**.
2. When the owner goes out of scope, the value is **dropped**.

A third rule you often hear quoted alongside these — *one mutable reference XOR
any number of immutable references* — is a **borrowing** rule, not an ownership
rule; it governs references, not owners (see `borrowing.md`).

Together these eliminate use-after-free, double-free, and data races without a garbage collector.

---

## Move Semantics

By default, assigning a value to another variable or passing it to a function **moves** ownership. The original binding
becomes invalid.

```rust
let s1 = String::from("hello");
let s2 = s1; // s1 is moved into s2
// println!("{}", s1); // ERROR: borrow of moved value
```

### Types that are `Copy`

Types that implement `Copy` are duplicated bit-by-bit on assignment instead of moved. All primitive integers, floats,
`bool`, `char`, and tuples/arrays of `Copy` types are `Copy`.

```rust
let x = 42;
let y = x; // x is Copy, so this is a copy, not a move
println!("{x}"); // fine
```

**The rule:** a type can implement `Copy` only if **all of its fields are `Copy`** and it has **no `Drop`
implementation** — the compiler rejects `impl Copy` otherwise. Owning heap data disqualifies a type (`String`, `Vec`,
and `Box<T>` — even `Box<u8>` — are never `Copy`), because a bitwise copy would give two owners of the same
allocation, i.e., a double-free.

### When to derive `Copy`

```rust
#[derive(Clone, Copy)]
struct Point { x: f64, y: f64 }
```

Only derive `Copy` when:

- The type is small (a few words).
- It doesn't own heap-allocated data.
- Copying is semantically equivalent to cloning (no custom `Clone` logic).

---

## Clone vs Copy

`Clone` is an explicit, potentially expensive deep copy. `Copy` is an implicit, cheap bitwise copy.

```rust
#[derive(Clone)] // NOT Copy — Vec owns heap data
struct Buffer {
    data: Vec<u8>,
}

let a = Buffer { data: vec![1, 2, 3] };
let b = a.clone(); // explicit deep copy
// let c = a; // would move, making `a` invalid
```

**Pitfall:** Deriving `Clone` on a type that contains a `String` or `Vec` will deep-copy the heap data. This is correct
but can be expensive. If you need cheap cloning, consider `Rc<T>` or `Arc<T>`.

---

## Partial Moves

When a value has multiple fields, you can move out individual fields while keeping the rest usable.

```rust
struct User {
    name: String,
    age: u32,
}

let user = User { name: String::from("Alice"), age: 30 };
let name = user.name; // partial move of `name`
// println!("{}", user.name); // ERROR: partially moved value
println!("{}", user.age); // fine — `age` is Copy
```

### Partial move from a function argument

```rust
fn take_name(User { name, .. }: User) -> String {
    name
}

let user = User { name: String::from("Bob"), age: 25 };
let n = take_name(user); // moves `name` out, `age` is dropped with the rest
```

### Restructuring to avoid partial moves

```rust
let User { name, age } = user; // destructure all fields at once
```

---

## Drop Semantics

When a value goes out of scope, its `Drop::drop` method is called automatically. This is where you release resources
(file handles, network sockets, locks).

```rust
struct Connection {
    url: String,
}

impl Drop for Connection {
    fn drop(&mut self) {
        println!("Closing connection to {}", self.url);
    }
}

{
    let conn = Connection { url: "db://localhost".into() };
    // conn dropped here — "Closing connection" printed
}
```

### Drop order

Per the Rust Reference ("Destructors"):

- **Local variables** in a block: **reverse declaration order** (last declared, first dropped).
- **Function parameters**: after the body's locals (they live in the outer function scope), in **reverse declaration
  order** among themselves. So `fn f(a: T, b: T) { let c = ...; }` drops `c`, then `b`, then `a`.
- **Struct fields** and **enum-variant fields**: **declaration order** (first field first).
- **Tuples**: in order (`.0` first).
- **Vec/array/slice elements**: first element to last.
- **Closure captures** (moved): **unspecified order** — don't rely on it.
- A value is dropped when its owner goes out of scope, not when the last reference to it is gone (that's `Rc`/`Arc`
  territory — see below).

```rust
struct A; struct B;
impl Drop for A { fn drop(&mut self) { println!("A"); } }
impl Drop for B { fn drop(&mut self) { println!("B"); } }

struct Pair { first: A, second: B } // drops A ("first"), then B

fn f(x: A, y: B) {
    let z = Pair { first: A, second: B };
} // drops: z.first, z.second, then y, then x
```

### `std::mem::drop`

Force early drop:

```rust
let lock = mutex.lock().unwrap();
// critical section
std::mem::drop(lock); // release immediately instead of at end of scope
```

### `ManuallyDrop`

Prevent automatic drop (for unsafe code that manages its own memory):

```rust
use std::mem::ManuallyDrop;

let mut v = ManuallyDrop::new(String::from("leak"));
// v will NOT be dropped — you must call `ManuallyDrop::drop` manually
```

---

## Ownership Boundaries

### Function boundaries

```rust
fn process(s: String) { /* takes ownership */ }
fn process_ref(s: &String) { /* borrows */ }
fn process_mut(s: &mut String) { /* mutable borrow */ }
```

### Struct boundaries

A struct owns its fields. When the struct is dropped, all fields are dropped.

```rust
struct Parser {
    input: String,   // owned
    pos: usize,      // Copy
}
```

### Trait object boundaries

```rust
trait Draw { fn draw(&self); }
struct Button;
impl Draw for Button { fn draw(&self) {} }

let elements: Vec<Box<dyn Draw>> = vec![Box::new(Button)];
// Box<dyn Draw> owns the heap-allocated trait object
```

---

## When to Use `Box<T>`

`Box<T>` is a smart pointer that heap-allocates a value. Use it when:

1. **Recursive types** — a type that contains itself indirectly needs indirection:

   ```rust
   enum List {
       Cons(i32, Box<List>),
       Nil,
   }
   ```

2. **Large values on the stack** — avoid blowing the stack with large structs:

   ```rust
   let big = Box::new([0u8; 1_000_000]);
   ```

3. **Trait objects** — `Box<dyn Trait>` for type erasure:

   ```rust
   fn make_drawable() -> Box<dyn Draw> { Box::new(Button) }
   ```

4. **Reducing enum size** — if one variant is much larger than the rest, boxing it shrinks the whole enum:

   ```rust
   enum Message {
       Quit,
       Move { x: i32, y: i32 },
       Payload(Box<[u8; 4096]>), // without Box, every Message would be 4+ KB
   }
   ```

   (A related but distinct fact: the **null-pointer optimization** makes
   `Option<Box<T>>` the same size as a bare `Box<T>` — `Box` can never be null,
   so `None` reuses the null bit pattern instead of needing a discriminant word.)

**When NOT to use `Box`:** For small, short-lived values. Stack allocation is faster and cache-friendly.

---

## When to Use `Rc<T>`

`Rc<T>` (reference counting) provides **single-threaded** shared ownership. Use it when:

- Multiple parts of your program need read-only access to the same data.
- You need a tree or graph structure with shared nodes.
- You want to avoid cloning large immutable data.

```rust
use std::rc::Rc;

let shared = Rc::new(String::from("shared data"));
let ref2 = Rc::clone(&shared); // increments refcount, cheap
let ref3 = Rc::clone(&shared);
println!("count: {}", Rc::strong_count(&shared)); // 3
```

**When NOT to use `Rc<T>`:**

- In multi-threaded code — use `Arc<T>` instead.
- When you need interior mutability — combine with `RefCell<T>`.
- When you can restructure to use borrowing instead.

---

## When to Use `Arc<T>`

`Arc<T>` (atomic reference counting) provides **multi-threaded** shared ownership. Use it when:

- Sharing immutable data across threads.
- Sharing configuration or read-only state in a server.

```rust
use std::sync::Arc;
use std::thread;

let config = Arc::new(AppConfig::default());
let mut handles = vec![];

for _ in 0..4 {
    let cfg = Arc::clone(&config);
    handles.push(thread::spawn(move || {
        cfg.connect();
    }));
}
```

**When NOT to use `Arc<T>`:**

- Single-threaded code — `Rc<T>` is cheaper (non-atomic refcount).
- When you need mutable shared state — use `Arc<Mutex<T>>` or `Arc<RwLock<T>>`.

---

## Reference Cycles and `Weak`

`Rc`/`Arc` keep a value alive while any **strong** reference exists. A cycle of
strong references (parent → child → parent) keeps every count above zero forever:
the memory leaks. Break cycles by making the back-reference a `Weak<T>`:

```rust
use std::cell::RefCell;
use std::rc::{Rc, Weak};

struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,   // weak: does not keep the parent alive
    children: RefCell<Vec<Rc<Node>>>,
}

let leaf = Rc::new(Node {
    value: 3,
    parent: RefCell::new(Weak::new()),
    children: RefCell::new(vec![]),
});
let branch = Rc::new(Node {
    value: 5,
    parent: RefCell::new(Weak::new()),
    children: RefCell::new(vec![Rc::clone(&leaf)]),
});
*leaf.parent.borrow_mut() = Rc::downgrade(&branch);
// branch -> leaf is strong, leaf -> branch is weak: no cycle, no leak.
```

Access a `Weak` via `.upgrade()`, which returns `Option<Rc<T>>` — the pointee may
already be gone.

**Leaks are safe.** Rust's ownership prevents use-after-free, not leaks:
`mem::forget`, `Box::leak`, and `Rc`/`Arc` cycles all leak memory, and all are
**safe** operations. Destructors are not guaranteed to run, so no type may rely
on its `Drop` impl running for memory *safety* (per the Rust Reference,
"Destructors").

---

## Common Pitfalls

### 1. Moving out of a borrowed value

```rust
fn bad(s: &String) -> String {
    *s // ERROR: cannot move out of `*s` which is behind a shared reference
}
```

**Fix:** Return a clone, or change the signature to take ownership.

### 2. Returning references to local variables

```rust
fn bad() -> &String {
    let s = String::from("temporary");
    &s // ERROR: `s` will be dropped
}
```

**Fix:** Return an owned value, or use `'static` lifetime for string literals.

### 3. `Clone` that doesn't deep-copy

```rust
#[derive(Clone)]
struct Config {
    data: Vec<u8>, // clone deep-copies the Vec — may be surprising
}
```

If you want cheap cloning, wrap in `Rc`/`Arc` or document the cost.

### 4. Dropping a value while borrowed

```rust
let r;
{
    let s = String::from("temp");
    r = &s;
} // s dropped here
// println!("{}", r); // ERROR: r points to dropped data
```

The borrow checker catches this at compile time.

---

## Summary Table

| Pattern | Use When | Thread-Safe | Overhead |
| --------- | ---------- | ------------- | ---------- |
| Direct ownership | Single owner, most common | N/A | Zero |
| `Box<T>` | Recursive types, large values, trait objects | Yes (if `T: Send`) | Heap alloc |
| `Rc<T>` | Shared ownership, single-threaded | No | Refcount |
| `Arc<T>` | Shared ownership, multi-threaded | Yes | Atomic refcount |
| `Cow<'a, T>` | Borrow-or-own, lazy cloning | If `T: Send + Sync` | Enum tag |
