---
name: rust-unsafe-checker
description: Use when writing or reviewing unsafe Rust or FFI.
---

# rust-unsafe-checker — Unsafe Rust Review

Unsafe Rust is a contract with the compiler: you take responsibility for invariants the compiler normally enforces. This
skill is the checklist for writing and reviewing that contract. Authority: [The
Rustonomicon](https://doc.rust-lang.org/nomicon/).

## When to use

- Writing or reviewing `unsafe` blocks, `unsafe fn`, `unsafe trait`, `unsafe impl`.
- FFI boundaries (C ABI, `extern "C"`, `repr(C)`).
- Raw pointers, `NonNull`, `MaybeUninit`, `static mut`.
- Performance code that reaches for unsafe (atomics, transmute, pointer arithmetic).

## The unsafe surface

Minimize it. Every `unsafe` block should be:

1. **Necessary** — safe Rust cannot express it (FFI, certain layouts, performance-critical primitives).
2. **Small** — wrap a minimal unsafe core in a safe API; never expose `unsafe` in the public API unless unavoidable.
3. **Documented** — a `// SAFETY:` comment stating the invariant the caller must uphold.
4. **Reviewed** — unsafe code gets extra scrutiny; Miri for validation.

## Raw pointers

- `*const T` / `*mut T` — raw pointers. Creating them is safe; dereferencing is `unsafe`.
- A raw pointer may be null, dangling, misaligned, or alias another pointer's data — the compiler makes no guarantees.
- Dereference only when you can prove: non-null, aligned, initialized, and valid for `T` for the entire access.
- Pointer arithmetic (`offset`, `add`, `sub`) is safe only within the bounds of the same allocation (or one past the end).
- `NonNull<T>` — a non-null raw pointer wrapper; use it in public APIs to encode non-nullness. `NonNull::new`
  returns `Option`; `new_unchecked` is unsafe.
- `null()`/`null_mut()` for FFI null sentinels; `as_ref()`/`as_mut()` are `unsafe fn` - they check for null and
  return `Option`, but the caller must still guarantee the pointee is valid, aligned, initialized, and lives for
  the returned lifetime (a dangling pointer passes the null check).

## MaybeUninit and initialization

- `MaybeUninit<T>` — a possibly-uninitialized `T`. Use it for partial initialization, FFI structs, and
  performance-sensitive construction.
- `MaybeUninit::uninit()` creates uninitialized memory; `assume_init()` is unsafe and valid only after full initialization.
- `write()` initializes without dropping the old value; `zeroed()` is unsafe (not all bit patterns are valid).
- Partial initialization: track which fields are initialized; never `assume_init` a struct with uninitialized fields.
- `ptr::write`, `ptr::read`, `ptr::copy`, `ptr::copy_nonoverlapping` — the safe-ish primitives for manual memory
  management; each has strict preconditions (alignment, non-overlap, validity).

## Layout, alignment, validity

- `size_of::<T>()`, `align_of::<T>()` — compile-time layout queries.
- `Layout::new::<T>()`, `Layout::array::<T>(n)` — for manual allocation; `alloc`/`dealloc` are unsafe and require
  the exact layout used.
- Validity invariants: a `bool` is 0 or 1; a `char` is a valid Unicode scalar; references/non-null pointers are
  aligned and non-null; enums have valid discriminants. Violating these is UB even if the code "works."
- `repr(C)` for FFI/ABI layout; `repr(transparent)` for single-field wrappers; `repr(align(N))` for alignment. Default
  `repr(Rust)` has no layout guarantee.
- `transmute` is almost never the answer — it reinterprets bits with no validity check. Prefer `From`/`TryFrom`,
  `ptr::read`, or explicit bit manipulation.

## Aliasing and provenance

- Rust's aliasing rules (Stacked Borrows / Tree Borrows, enforced by Miri): a `&mut T` must be the only live path to
  its data; `&T` may alias other `&T` but not `&mut T`.
- Creating a `&mut` from a `&` (or vice versa) invalidates the original reference — even creating a raw pointer from
  a `&` and then writing through it after the `&` is used again is UB.
- `addr_of!`/`addr_of_mut!` create raw pointers without intermediate references — use them when you need a pointer
  without forming a reference.
- Provenance: pointers carry permission to access; `expose_provenance`/`with_exposed_provenance` (1.84+) for FFI
  interop. Do not round-trip pointers through integers.

## FFI and ABI

- `extern "C" fn` — the C ABI calling convention. `#[no_mangle]` + `pub extern "C"` for exported symbols.
- `#[repr(C)]` structs for C-compatible layout; `#[repr(C, packed)]` only when the C header demands it (unaligned
  reads are UB).
- `CString`/`CStr` for string boundaries — never pass `&str` directly; `CString::new` fails on interior nulls.
- `catch_unwind` at FFI boundaries — a Rust panic unwinding into C is UB.
- `Send`/`Sync` across FFI: `extern "C"` functions are not automatically thread-safe; document thread-safety contracts.
- `bindgen` for generating Rust bindings from C headers; `cbindgen` for the reverse.

## Send, Sync, and unsafe traits

- `unsafe trait` — the trait itself has invariants implementors must uphold (e.g., `Send`, `Sync`, `TrustedLen`).
- `unsafe impl` — you assert the type upholds the trait's invariant. `unsafe impl Send for MyType` is a promise that
  `MyType` is safe to move across threads; `unsafe impl Sync` that it's safe to share.
- `Send` = safe to move to another thread. `Sync` = safe to share `&T` across threads. `RefCell<T>` is `!Sync`
  (runtime borrow checking is not thread-safe) but `Send` when `T: Send`. `Rc<T>` is neither.
- `static mut` - restricted in edition 2024: `static_mut_refs` (taking references to a `static mut`) is
  deny-by-default (an error unless narrowly `#[allow]`ed); take raw pointers with `&raw const`/`&raw mut` instead,
  or use `AtomicUsize`, `OnceLock`, or `Mutex`.

## unsafe_op_in_unsafe_fn and edition 2024

- Edition 2024 makes `unsafe_op_in_unsafe_fn` a lint (deny by default): an `unsafe fn` no longer implies an `unsafe`
  block. Write explicit `unsafe {}` blocks inside `unsafe fn` — this is the correct pattern in all editions.
- Edition 2024 also: `gen` blocks (unstable), `unsafe extern` blocks, `#[unsafe(no_mangle)]` attribute syntax. Prefer
  edition 2021 unless the project opts into 2024.

## SAFETY comments

Every `unsafe` block must have a `// SAFETY:` comment stating:

```rust
// SAFETY: `ptr` is non-null, aligned, and points to a valid, initialized `T`
// for the duration of this call, and no other references alias it.
unsafe { *ptr = value; }
```

The comment is the contract. If you cannot write a convincing SAFETY comment, the unsafe is not justified.

## Miri and testing unsafe

- [Miri](https://github.com/rust-lang/miri) — an interpreter that detects UB (aliasing violations, invalid values,
  uninitialized reads). Run `cargo +nightly miri test` on unsafe code.
- Miri is slow and requires nightly; run it on the unsafe core, not the whole workspace.
- Test unsafe invariants: null pointers, misaligned access, partial initialization, aliasing violations — Miri
  catches what the compiler cannot.
- `cargo fuzz` (libFuzzer) for fuzzing unsafe parsers and FFI boundaries.
- Property tests for unsafe data structures (invariants hold under arbitrary operation sequences).

## Review checklist

- [ ] Is the unsafe necessary? Can safe Rust express this?
- [ ] Is the unsafe surface minimal and wrapped in a safe API?
- [ ] Does every `unsafe` block have a `// SAFETY:` comment?
- [ ] Are raw pointers non-null, aligned, initialized, and valid for `T`?
- [ ] Are aliasing rules respected (no `&mut` aliasing, no invalidation)?
- [ ] Is initialization complete before `assume_init`?
- [ ] Are FFI types `repr(C)` and ABI-correct?
- [ ] Are `Send`/`Sync` implications of `unsafe impl` justified?
- [ ] Is `static mut` avoided (edition 2024)?
- [ ] Has Miri been run on the unsafe core?
- [ ] Are unsafe traits/impls documented with their invariants?

## House rules

- Native first: `std` before crates.
- No blocking in async; no casual `unwrap()` in production paths.
- Version-sensitive advice names its version.
- Examples use edition 2021.
