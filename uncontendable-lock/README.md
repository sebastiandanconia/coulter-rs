# uncontendable-lock

A lock for code that must never contend.

When your design says a critical section is reachable from only one execution context at a time, put this lock on it. Under the `enabled` feature, a second holder does not wait: it prints both stacks and kills the process. With the feature off, the type is a zero-sized no-op.

That is the whole product. Use it as a runtime assertion about concurrency structure, not as a mutex.

```rust
use uncontendable_lock::UncontendableLock;

let lock = UncontendableLock::new();
let _guard = lock.lock();
// A second lock() here aborts when the crate feature `enabled` is on.
```

## Why this exists

Concurrent bugs are cheap to introduce and expensive to see. The usual tools (a real mutex, a channel, an actor) change the design. `UncontendableLock` does not: it is a tripwire you leave on a path that is *supposed* to be exclusive.

Typical placements:

- A shard or partition that "only the owner thread touches"
- A state machine advanced by one worker, with tests that deliberately race callers
- FFI or legacy boundaries where the docs say "caller must serialize" and you want teeth
- Data-pipeline stages that claim single-reader / single-writer structure

The idea comes from the "Testing Concurrent Code" talk at JavaOne 2007: *use a lock when your design says you do not need one, to verify that claim at runtime.*

## Quick start

```toml
# Cargo.toml — dependency stays inert unless you opt in.
[dependencies]
uncontendable-lock = "0.1"

# Workspace or binary crate: turn checks on for debug/test builds.
[features]
# Example: wire it to your own debug feature.
debug-concurrency = ["uncontendable-lock/enabled"]
```

```rust
use uncontendable_lock::UncontendableLock;

struct Shard {
    // Present in release builds too; free when the feature is off.
    gate: UncontendableLock,
    // ...
}

impl Shard {
    fn mutate(&self) {
        let _guard = self.gate.lock();
        // Exclusive work.
    }
}
```

Enable during development:

```bash
cargo test --features enabled
cargo run --features enabled --bin my_app
```

Or from a parent crate:

```toml
[dependencies]
uncontendable-lock = { version = "0.1", features = ["enabled"] }
```

A common pattern is to enable it only for non-release profiles via the parent crate's feature flags, so production stays zero-cost without scattering `cfg` through business logic.

## Behavior

| Build                         | `lock()`                                         | Size of lock / guard |
|-------------------------------|--------------------------------------------------|----------------------|
| Default (feature off)         | No-op; overlapping guards are allowed            | 0 / 0 bytes          |
| `--features enabled`          | `try_lock`; on failure, print stacks and `exit(1)` | non-zero            |

With `enabled`:

1. Acquisition uses `std::sync::Mutex::try_lock` (never blocks).
2. Success stores a `std::backtrace::Backtrace` of the owner (`force_capture`, so it does not depend on `RUST_BACKTRACE`).
3. Failure writes a diagnostic to stderr and exits with status `1`:

```text
Error: UncontendableLock locked more than once:
First by:
   0: my_crate::owner_path
             at src/owner.rs:42:5
   1: ...
Then by:
   0: my_crate::contender_path
             at src/other.rs:17:5
   1: ...
```

4. Same-thread re-entry counts as contention (`std` mutexes are not re-entrant). That is intentional: a design that "does not need a lock" also does not recurse into itself while holding one.
5. Poisoned mutexes (prior owner panicked) are recovered so a debug session can continue; the tool is about concurrent holders, not panic safety of your data.

Without `enabled`, both `UncontendableLock` and `UncontendableGuard` are zero-sized. There is no runtime flag, no atomic, no allocation.

## API surface

Small on purpose:

- `UncontendableLock::new()` / `Default`
- `UncontendableLock::lock() -> UncontendableGuard<'_>`
- Guard is `#[must_use]`; drop releases (when enabled)

No `try_lock` for callers, no poisoning API, no async variant. If you need to wait, you want `std::sync::Mutex` or a better design. If you need async, put the tripwire at the sync boundary that still exists underneath.

```rust
use uncontendable_lock::UncontendableLock;

fn exclusive(lock: &UncontendableLock) {
    let _guard = lock.lock();
    // ...
} // released here
```

## Integration patterns

**Library authors.** Depend on the crate without enabling the feature. Document that downstream bins/tests may turn on `uncontendable-lock/enabled`. Your published code stays zero-cost; integrators get the tripwire when they ask for it.

**Application / data platform code.** Enable in CI and local debug profiles. Keep it off in release unless you explicitly want production fail-fast (usually you do not: aborting is correct for "this is impossible," wrong for "degrade and page").

**Tests that must observe a failure.** Contention calls `process::exit(1)`, which ends the process. Assert on it from a subprocess (see `tests/subprocess.rs` and the bins under `src/bin/`), not from an in-process `#[test]`.

**Where to put the lock.** Prefer the smallest scope that captures the invariant ("this shard's map is never shared") over wrapping an entire service. The backtraces are only as useful as the lock placement.

## Design notes

**Why abort instead of panic?** A panic can be caught. Contended "impossible" access is a failed invariant: fail the process, leave the stacks on stderr, do not let a `catch_unwind` in a thread pool paper over it. `exit(1)` is used rather than `abort()` so the diagnostic is flushed.

**Why a feature flag instead of `debug_assertions`?** So you can turn it on in a release-mode stress test, and turn it off in a debug build of a binary that cannot pay for backtraces. Parents can still wire the feature to `debug_assertions` if that is their policy.

**Cost when enabled.** One `try_lock` per acquisition, plus a full backtrace capture on each successful lock and on contention. That is fine for tests and debug runs; it is not free. Keep the feature off in hot production paths.

**Race on the owner backtrace.** The owner stack is stored under a second mutex so a contender can still read it after `try_lock` fails. If the owner unlocks in the gap between those two steps, the diagnostic may say the owner backtrace was unavailable. This is acceptable for a debugging tool.

**No external dependencies.** `std` only. Easy to vendor, easy to audit, no supply-chain surface.

## Building and testing

```bash
# Zero-cost path (default): unit tests for no-op behavior and ZST layout
cargo test

# Tripwire path: unit tests + subprocess abort tests
cargo test --features enabled

# Manual inspection of the diagnostic
cargo run --features enabled --bin double_lock
cargo run --features enabled --bin cross_thread_lock

cargo clippy --all-targets --features enabled -- -D warnings
cargo doc --features enabled --no-deps --open
```

Bins (`single_lock`, `double_lock`, `cross_thread_lock`) exist only as fixtures for the subprocess tests and for manual inspection. They require `--features enabled`.

## Project layout

```text
uncontendable-lock/
  src/lib.rs                 Public API
  src/bin/                   Abort fixtures used by integration tests
  tests/subprocess.rs        Asserts exit status + diagnostic text
  Cargo.toml
  README.md
```

## License

[BSD-3-Clause](LICENSE). Copyright (c) 2021-2026 James Walker Crofts.

Permissive and GPL-compatible (including GPL-2.0), so this crate can be used from the parent tree or from proprietary pipelines without a special exception. Independent of the monorepo's GPL-2.0 license.

## Status

- Rust 2021, MSRV 1.65 (`const Mutex::new`, `std::backtrace`)
- API is small; 0.1.x may still tighten docs and diagnostics, not the shape of `lock()`

## Author

James Walker Crofts — software and data engineering; concurrency instrumentation ported from production debugging practice, not from a framework fashion cycle.
