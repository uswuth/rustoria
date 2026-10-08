---
name: rust-std
description: Use when choosing or using Rust standard-library APIs.
---

# rust-std — Standard Library API Discovery

The standard library is the foundation. Before reaching for a crate, check whether `std` already provides the capability
with the right semantics. This skill is a decision map, not an encyclopedia — for full API details consult [std
docs](https://doc.rust-lang.org/std/).

## When to use

- Choosing between `std` types (Vec vs VecDeque, HashMap vs BTreeMap, String vs &str).
- Picking the right iterator method or combinator.
- Deciding between Option/Result APIs.
- Selecting interior mutability (Cell vs RefCell vs OnceLock vs LazyLock).
- Choosing synchronization primitives (Mutex vs RwLock vs atomics).
- Working with fs/io/path/process/net without dependencies.
- Deciding which conversion trait to implement (From/TryFrom/AsRef/Borrow/Deref).

## Collections

| Need | Use | Notes |
| ------ | ----- | ------- |
| Growable sequence | `Vec<T>` | Default choice; contiguous, cache-friendly |
| Push/pop at both ends | `VecDeque<T>` | Ring buffer; O(1) at both ends |
| Rarely: middle insert/remove | `LinkedList<T>` | Almost never wins; cache-unfriendly |
| Ordered map | `BTreeMap<K,V>` | Sorted keys, range queries |
| Fast key lookup | `HashMap<K,V>` | O(1) avg; needs `Hash + Eq`; default `RandomState` (SipHash, randomized per process) is the DoS-resistant choice for attacker-controlled keys - a fixed hasher (`rustc-hash`'s `FxHashMap`) is faster but deterministic, so use it only for trusted keys when profiling shows hashing is hot |
| Ordered set | `BTreeSet<T>` | Sorted, range queries |
| Fast membership | `HashSet<T>` | O(1) avg |
| FIFO queue | `VecDeque<T>` or `std::sync::mpsc` | For cross-thread, use channels |
| LIFO stack | `Vec<T>` | `push`/`pop` |
| Priority queue | `BinaryHeap<T>` | Max-heap; `Reverse<T>` for min-heap |
| Fixed-size known at compile time | `[T; N]` or arrays | Stack-allocated, no heap |
| Slice view | `&[T]` | Borrowed view into Vec/array/other slices |

HashMap patterns: use `entry` API for insert-or-update (`map.entry(k).or_insert_with(...)`); `get` returns `Option<&V>`;
`remove` returns `Option<V>`. Iteration order is unspecified — never depend on it.

## Strings and text

- `String` — owned, growable, UTF-8. `&str` — borrowed slice. Pass `&str` into functions, return `String`.
- `String::with_capacity(n)` when size is known; `push_str` to append; `format!` for composition.
- `&str` methods: `split`, `split_whitespace`, `trim`, `starts_with`/`ends_with`, `contains`, `replace`,
  `parse::<T>()` (via `FromStr`), `chars`, `lines`.
- `char` is a Unicode scalar value (4 bytes). Iterate grapheme clusters only with the `unicode-segmentation` crate.
- `OsString`/`OsStr` for OS-native paths that may not be UTF-8.

## Option and Result

- `Option<T>` — value may be absent. `Result<T, E>` — operation may fail.
- Propagate with `?` — works in functions returning `Option` or `Result`, and `fn main() -> Result<(), E: Debug>` is stable.
- Combinators: `map`, `and_then` (flatMap), `or_else`, `map_or`/`map_or_else`, `ok_or`/`ok_or_else`, `as_deref`,
  `transpose` (Option<Result> ↔ Result<Option>), `flatten` (Option<Option>/Result<Result>), `inspect`/`inspect_err`
  (1.76+, log without consuming).
- `unwrap()`/`expect()` only in tests, demos, or provably-infallible cases. `unwrap_or`/`unwrap_or_else` for defaults.
- `?` also works in functions returning `Option`.

## Iterators

Iterators are lazy — nothing happens until a consuming call. Chain transformations without intermediate `collect()`
unless you need to reuse the result.

Consuming calls: `collect`, `sum`, `count`, `min`/`max`, `find`, `any`/`all`, `fold`, `for_each`, `nth`, `last`,
`position`, `partition`.

Transformation methods: `map`, `filter`, `flat_map`/`flatten`, `filter_map` (map + filter in one), `enumerate`, `zip`,
`skip`/`take`, `skip_while`/`take_while`, `step_by`, `cloned`/`copied`, `scan`, `inspect`, `by_ref` (for
partial consumption). There is no `Iterator::dedup` - dedup consecutive items by collecting into a `Vec` and calling
`Vec::dedup`, or use `itertools`.

`collect` targets: `Vec`, `String` (from `char` or `&str` iterators), `HashMap`/`BTreeMap`/`HashSet`/`BTreeSet` (from
pairs), `Result<Vec<T>, E>` (fail-fast collection), `Option<Vec<T>>`.

## Conversion and operator traits

| Trait | Direction | Use |
| ------- | ----------- | ----- |
| `From<T> for U` | infallible T→U | Implement on your type for ergonomic `U::from(t)`; also gives you `Into<U>` for free |
| `Into<U> for T` | infallible | Blanket-implemented from `From`; implement `From`, not `Into` |
| `TryFrom<T>`/`TryInto<U>` | fallible | When conversion can fail (e.g. `u64`→`u32` range check) |
| `AsRef<T>` | cheap borrow | Generic over `&str`/`String`, `&[T]`/`Vec<T>`; function parameters |
| `AsMut<T>` | cheap mutable borrow | Same, mutable |
| `Borrow<T>` | equivalence-preserving borrow | HashMap keys (`Borrow<str>` lets `HashMap<String,_>` be queried with `&str`) |
| `Deref`/`DerefMut` | smart-pointer coercion | Only for smart pointers (Box, Rc, Arc, Cow) — not for newtypes; prefer `AsRef` or an explicit accessor |
| `FromStr` | parse from string | Enables `"42".parse::<T>()`; implement for domain types |
| `Default` | zero value | Derive when all fields have sensible defaults |
| `PartialOrd`/`Ord` | ordering | Derive for data types; implement for domain ordering |
| `Hash` | hashing | Derive; must be consistent with `Eq` |
| `Display`/`Debug` | formatting | `Display` for user-facing, `Debug` for diagnostics; derive `Debug` always |
| `Index`/`IndexMut` | indexing | Implement for collection-like types |
| `Add`/`Sub`/etc. | operators | Implement for domain types where arithmetic is meaningful |

Rule of thumb: implement `From` (not `Into`), `AsRef` for borrow-generic parameters, `TryFrom` for fallible conversions,
`FromStr` for parsing, `Display` for user output.

## Smart pointers and interior mutability

| Type | Use when |
| ------ | ---------- |
| `Box<T>` | Heap allocation, recursive types, trait objects, large moves |
| `Rc<T>` | Single-threaded shared ownership |
| `Arc<T>` | Multi-threaded shared ownership (Send+Sync) |
| `Weak<T>` | Break Rc/Arc cycles (parent→child uses Weak) |
| `Cell<T>` | Interior mutability for `Copy` types; no borrow checking; not Sync |
| `RefCell<T>` | Interior mutability for non-Copy; runtime borrow checking (panics on violation); `!Sync` |
| `OnceCell<T>`/`OnceLock<T>` | Write-once initialization (thread-safe for OnceLock) |
| `LazyLock<T>` | Lazy static initialization (std, 1.80+; replaces `lazy_static!` crate) |
| `Mutex<T>`/`RwLock<T>` | Cross-thread mutual exclusion (see concurrency reference) |

Decision: single-thread + Copy → Cell; single-thread + non-Copy → RefCell; multi-thread → Mutex/RwLock/Arc;
one-time init → OnceLock; lazy static → LazyLock.

## Synchronization (std)

- `Mutex<T>` — exclusive access; guard held across `.await` is a compile error in spawned tasks (!Send) — scope guards tightly.
- `RwLock<T>` — many readers or one writer; prefer Mutex unless read-heavy.
- Atomics (`AtomicUsize` etc.) — lock-free counters/flags; default `SeqCst`; see concurrency reference for ordering.
- `Condvar` — wait/notify on a Mutex guard; always re-check the condition in a loop (spurious wakeups).
- `Barrier` — rendezvous point for N threads.
- `mpsc` channels - std gives only `std::sync::mpsc::{channel, sync_channel}` (unbounded/bounded); `oneshot`,
  `broadcast`, and `watch` channels are runtime crates (`tokio::sync`), a justified dependency when std's
  single-producer multi-consumer channel does not fit.
- `thread::scope` (1.63+) — scoped threads can borrow stack data safely.
- `thread::available_parallelism()` — size thread pools to CPU count.
- Mutex poisoning: `lock()` returns `LockResult`; a panic while holding the lock poisons it. `.unwrap()` propagates
  the panic; `into_inner()` recovers if poisoning is acceptable.

## fs, io, path

- `Path`/`PathBuf` — path manipulation (join, parent, extension, file_name); `PathBuf` for owned, `Path` for borrowed.
- `fs::read_to_string`, `fs::read`, `fs::write` — whole-file ops.
- `File` + `BufReader`/`BufWriter` — buffered streaming; `BufRead::lines()` for line iteration.
- `OpenOptions` — fine-grained open flags (read/write/append/create/truncate).
- `fs::canonicalize` — resolve to absolute canonical path (security checks: verify prefix after canonicalization).
- `read_dir` — directory iteration; `DirEntry::path`, `file_type`, `metadata`.
- `io::ErrorKind` — match on `NotFound`, `PermissionDenied`, `AlreadyExists` etc. for specific handling.
- `stdin`/`stdout`/`stderr` — `io::stdin().lock()` for line input.

## process, env, net

- `std::env` — `var`, `var_os`, `args`, `current_dir`, `set_var` (unsafe in edition 2024), `temp_dir`.
- `Command` — spawn processes; pass args as an array (never build shell strings — argument injection); `Stdio` for
  piping; `output()` for captured output; `status()` for exit code; `Child` for stdin/stdout interaction.
- `TcpListener`/`TcpStream` — blocking TCP; `UdpSocket` for UDP; `SocketAddr`/`SocketAddrV4`/`SocketAddrV6` for addresses.
- `ToSocketAddrs` — "host:port" parsing for connect/bind.
- For async I/O use tokio (see rust-tokio); std net is blocking.

## time, panic, misc

- `Duration` — time spans; `Instant` — monotonic clock (measurements); `SystemTime` — wall clock (may jump; use
  for timestamps, not intervals).
- `thread::sleep` — blocking sleep (never in async).
- `panic!`/`catch_unwind` — panics unwind by default; `catch_unwind` for FFI boundaries or thread isolation; `panic
  = "abort"` in release profiles for no-unwind builds.
- `std::any::type_name` — runtime type name for diagnostics.
- `std::mem` — `size_of`/`align_of`, `replace`, `take`, `swap`, `forget` (leaks memory — safe but usually wrong).
- `std::ffi` — `CString`/`CStr` for FFI boundaries; `OsString` for OS strings.
- `std::marker` — `PhantomData` for unused generics/ownership hints, `PhantomPinned`.

## Native-first rule

The decision chain: **NATIVE FIRST -> VERIFY NEED -> CHOOSE DEPENDENCY -> VERIFY MAINTENANCE.**

Before adding a dependency:

1. Does `std` (or Cargo, or rustc/rustdoc) already provide this?
2. Does `std`'s semantics/performance suffice?
3. What does the crate add beyond `std`?
4. Weigh maintenance, security, MSRV, compile time, binary size, maturity, ops cost.
5. Add the dependency when it provides justified value: materially more capability, ergonomics,
   portability, performance, ecosystem compatibility, or domain functionality that std does not
   reasonably provide.

This is not "never use dependencies" - it is "understand native capabilities first, then introduce dependencies
intentionally." See `rust-production` for the full rule.

### Capability decision table

What std covers per capability, and when a dependency is the right call:

| Capability | Native first | Dependency justified when |
| ---------- | ------------ | -------------------------- |
| Collections | `Vec`, `HashMap`, `BTreeMap`, `VecDeque`, `HashSet`, `BTreeSet`, `BinaryHeap` | insertion-order maps (`indexmap`), profiled allocation pressure (`smallvec`), concurrent maps (usually `RwLock<HashMap>` first, `dashmap` only when profiling proves it) |
| Strings | `String`/`str`, `format!`/`write!`, `Cow` | grapheme-cluster operations (`unicode-segmentation` - std `chars()` yields code points, not user-perceived characters) |
| Iterators | `std::iter` combinators, `slice::chunks` | iterator-level chunking/dedup/grouping (`itertools`); std has no `Iterator::dedup` |
| Conversions | `From`/`Into`, `TryFrom`/`TryInto`, `AsRef`/`AsMut` | almost never - the newtype pattern covers domain conversions |
| Parsing | `str::parse`, `FromStr`, `split`, `char` methods | no std regex (use `regex`), no URL parsing (`url`), grammars (`nom`), dates (`chrono`/`time`) |
| Errors | `Result`/`Option`, `?`, custom error enums, `Box<dyn Error>` | large codebases: `thiserror` (library derive) / `anyhow` (application context) are ergonomics wins, not capability wins |
| Filesystem | `std::fs`, `Path`/`PathBuf`, `read_dir` | file watching (`notify`), async (`tokio::fs`), skip/hidden-aware walks (`walkdir`/`ignore`) |
| IO | `Read`/`Write`, `BufReader`/`BufWriter`, stdio | compression (`flate2`), shared byte buffers in network code (`bytes`), async (`tokio::io`) |
| Networking | `TcpStream`/`TcpListener`/`UdpSocket` (blocking) | async (`tokio::net`), TLS (`rustls`), HTTP (`hyper`/`axum`/`reqwest`) |
| Processes | `std::process::Command`, piped stdio | command timeouts (std has none: manage a kill thread or `wait-timeout`), cross-platform process info (`sysinfo`) |
| Synchronization | `Mutex`, `RwLock`, `Condvar`, `Barrier`, `OnceLock`, `LazyLock`, atomics | locks held across `.await` (`tokio::sync`), cross-process file locks (`fd-lock`/`fs2`) |
| Concurrency | threads, `thread::scope` (1.63+), `Arc`, atomics | data parallelism (`rayon`), async runtime (`tokio`) |
| Channels | `std::sync::mpsc::{channel, sync_channel}` | `oneshot`/`broadcast`/`watch` (`tokio::sync`), MPMC with backpressure (`flume`) |
| Time | `Instant`, `SystemTime`, `Duration` | async timers (`tokio::time`), calendars/timezones/formatting (`chrono`/`time` - std cannot format dates) |
| Paths | `std::path`, `components()`, `file_name()` | pattern expansion (`glob`), UTF-8 paths for interop (`camino`) |
| Environment | `std::env` (`set_var` is unsafe in edition 2024) | loading `.env` files in dev (`dotenvy`, dev-dependency only) |
| Testing | `cargo test`, `#[test]`, `should_panic`, doc tests | properties (`proptest`), snapshots (`insta`), process isolation (`cargo-nextest`) |
| Documentation | rustdoc, intra-doc links, doc tests, `cargo doc --no-deps` | essentially never for a crate; a docs site generator only for large sites |
| Benchmarking | `std::hint::black_box` + `Instant`, `harness = false` benches | statistics, warmup/outlier handling, regression detection (`criterion`) |
| Workspace management | Cargo workspaces, `[workspace.dependencies]`, `cargo metadata` for tooling | unused-dependency detection (`cargo-machete`/`cargo-udeps`) |
| Dependency management | `cargo add`/`remove`/`update`/`tree`/`metadata` | security/policy checks (`cargo-audit`, `cargo-deny`, `cargo vet`) - see `rust-dependencies` |
