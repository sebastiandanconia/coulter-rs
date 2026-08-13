//! A lock that must never see contention.
//!
//! `UncontendableLock` is a debugging tool for designs that claim a critical
//! section is reachable from only one execution context at a time. Enable the
//! `enabled` feature and every acquisition records a backtrace; a second
//! concurrent (or re-entrant) acquisition prints both backtraces and exits the
//! process.
//!
//! When the feature is off, the lock and its guard are zero-sized no-ops: no
//! atomics, no allocations, nothing in the generated code beyond what the
//! caller already does with the guard binding.
//!
//! # When to use it
//!
//! - A protocol, actor, or shard model says "this state is never shared."
//! - You want a runtime tripwire while testing concurrent code, not a real mutex.
//! - You would rather fail fast with two stacks than debug silent corruption.
//!
//! # When not to use it
//!
//! - As a general-purpose mutex. It does not wait; it dies.
//! - In production paths you cannot afford to abort. Gate it on the feature
//!   (and typically on `debug_assertions` at the call site or in your
//!   workspace `Cargo.toml`).
//!
//! # Example
//!
//! ```
//! use uncontendable_lock::UncontendableLock;
//!
//! let lock = UncontendableLock::new();
//! {
//!     let _guard = lock.lock();
//!     // Exclusive work. A second lock() here would abort when `enabled`.
//! }
//! let _guard = lock.lock(); // Fine: the previous guard was dropped.
//! ```
//!
//! # Feature flag
//!
//! | Feature   | Effect                                              |
//! |-----------|-----------------------------------------------------|
//! | (none)    | Zero-cost no-op (default)                           |
//! | `enabled` | Record owner backtraces; abort process on contention |
//!
//! # Origin
//!
//! Inspired by the "Testing Concurrent Code" talk at JavaOne 2007: "use a lock
//! when your design says you do not need one, to verify that claim at runtime.

#![cfg_attr(docsrs, feature(doc_cfg))]

use std::fmt;

#[cfg(feature = "enabled")]
use std::backtrace::Backtrace;
#[cfg(feature = "enabled")]
use std::sync::{Mutex, MutexGuard, TryLockError};

/// A lock that aborts the process if it is ever contended.
///
/// See the [crate-level documentation](crate) for intent, feature flags, and
/// examples.
///
/// Both `UncontendableLock` and [`UncontendableGuard`] are `Send + Sync` when
/// that is meaningful; with the feature disabled they are zero-sized.
pub struct UncontendableLock {
    #[cfg(feature = "enabled")]
    inner: Mutex<()>,
    /// Backtrace of the current owner, if any.
    ///
    /// Stored under a separate mutex so a contending thread can still read it
    /// after `try_lock` fails. There is a narrow race if the owner unlocks
    /// between the failed `try_lock` and this read; the diagnostic then reports
    /// that the owner backtrace was unavailable. Acceptable for a debug tool.
    #[cfg(feature = "enabled")]
    owner_bt: Mutex<Option<String>>,
}

impl UncontendableLock {
    /// Creates a new unlocked `UncontendableLock`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            #[cfg(feature = "enabled")]
            inner: Mutex::new(()),
            #[cfg(feature = "enabled")]
            owner_bt: Mutex::new(None),
        }
    }

    /// Acquires the lock, returning an RAII guard.
    ///
    /// With the `enabled` feature:
    /// - On success, stores a backtrace of the acquiring stack.
    /// - On contention (including re-entrant acquisition on the same thread),
    ///   prints the owner and contender backtraces to stderr and exits with
    ///   status `1`. Destructors on other threads are not run.
    ///
    /// Without the feature, returns immediately with a zero-sized guard.
    ///
    /// # Examples
    ///
    /// ```
    /// use uncontendable_lock::UncontendableLock;
    ///
    /// let lock = UncontendableLock::new();
    /// let guard = lock.lock();
    /// drop(guard);
    /// ```
    #[inline]
    #[must_use = "if unused, the lock is immediately released"]
    pub fn lock(&self) -> UncontendableGuard<'_> {
        #[cfg(feature = "enabled")]
        {
            let guard = match self.inner.try_lock() {
                Ok(g) => g,
                // Prior owner panicked while holding the lock. Recover the
                // mutex so a debug session can continue past that panic; the
                // uncontendable invariant is about concurrent holders, not
                // panic safety of the protected data.
                Err(TryLockError::Poisoned(p)) => p.into_inner(),
                Err(TryLockError::WouldBlock) => {
                    contend_and_exit(self);
                }
            };

            *self.owner_bt.lock().unwrap_or_else(|e| e.into_inner()) =
                Some(capture_backtrace());

            UncontendableGuard {
                lock: self,
                _guard: guard,
            }
        }

        #[cfg(not(feature = "enabled"))]
        UncontendableGuard {
            _marker: std::marker::PhantomData,
        }
    }
}

impl Default for UncontendableLock {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for UncontendableLock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("UncontendableLock");
        #[cfg(feature = "enabled")]
        {
            // Best-effort: true if another thread holds the lock right now.
            d.field("held", &self.inner.try_lock().is_err());
        }
        #[cfg(not(feature = "enabled"))]
        {
            d.field("enabled", &false);
        }
        d.finish()
    }
}

/// RAII guard returned by [`UncontendableLock::lock`].
///
/// Dropping the guard releases the lock (when the `enabled` feature is active).
/// The guard is `#[must_use]`; ignoring it releases the lock immediately.
#[must_use = "if unused, the lock is immediately released"]
pub struct UncontendableGuard<'a> {
    #[cfg(feature = "enabled")]
    lock: &'a UncontendableLock,
    #[cfg(feature = "enabled")]
    _guard: MutexGuard<'a, ()>,
    #[cfg(not(feature = "enabled"))]
    _marker: std::marker::PhantomData<&'a ()>,
}

#[cfg(feature = "enabled")]
impl Drop for UncontendableGuard<'_> {
    fn drop(&mut self) {
        *self.lock.owner_bt.lock().unwrap_or_else(|e| e.into_inner()) = None;
        // `_guard` unlocks after this method returns.
    }
}

impl fmt::Debug for UncontendableGuard<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UncontendableGuard").finish_non_exhaustive()
    }
}

/// Captures a multi-line backtrace via [`Backtrace`]'s `Display` impl.
///
/// `force_capture` ignores `RUST_BACKTRACE`; the frames are always present so
/// the diagnostic is useful under test harnesses that clear that variable.
///
/// Leading frames inside this crate's lock machinery are stripped so the first
/// printed frame is the caller's code.
#[cfg(feature = "enabled")]
fn capture_backtrace() -> String {
    trim_internal_frames(&Backtrace::force_capture().to_string())
}

/// Symbol prefixes for frames that are noise in a contention report.
#[cfg(feature = "enabled")]
const INTERNAL_FRAME_PREFIXES: &[&str] = &[
    "uncontendable_lock::capture_backtrace",
    "uncontendable_lock::contend_and_exit",
    "uncontendable_lock::trim_internal_frames",
    "uncontendable_lock::UncontendableLock::lock",
];

/// Drops leading internal frames and renumbers the rest from 0.
///
/// `Backtrace`'s `Display` format is stable enough for this: each frame is a
/// symbol line (`   N: path`) optionally followed by an `at` location line.
#[cfg(feature = "enabled")]
fn trim_internal_frames(bt: &str) -> String {
    let lines: Vec<&str> = bt.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim_start();
        // Symbol lines look like "0: foo::bar" after trimming leading spaces.
        if let Some(rest) = line.split_once(':').map(|(_, r)| r.trim_start()) {
            let is_internal = INTERNAL_FRAME_PREFIXES
                .iter()
                .any(|p| rest.starts_with(p));
            if is_internal {
                i += 1;
                // Skip the following "at" location line when present.
                if i < lines.len() && lines[i].trim_start().starts_with("at ") {
                    i += 1;
                }
                continue;
            }
        }
        break;
    }

    let remaining = &lines[i..];
    if remaining.is_empty() {
        return bt.to_owned();
    }

    let mut out = String::with_capacity(bt.len());
    let mut frame_no = 0usize;
    let mut idx = 0usize;
    while idx < remaining.len() {
        let line = remaining[idx];
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.split_once(':') {
            // Rewrite "   N: symbol" -> "   frame_no: symbol"
            if rest.0.trim().chars().all(|c| c.is_ascii_digit()) {
                let symbol = rest.1;
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(&format!("   {frame_no}:{symbol}"));
                frame_no += 1;
                idx += 1;
                if idx < remaining.len() && remaining[idx].trim_start().starts_with("at ") {
                    out.push('\n');
                    out.push_str(remaining[idx]);
                    idx += 1;
                }
                continue;
            }
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(line);
        idx += 1;
    }
    out
}

/// Prints the contention diagnostic and terminates the process.
#[cfg(feature = "enabled")]
fn contend_and_exit(lock: &UncontendableLock) -> ! {
    let owner = lock
        .owner_bt
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .unwrap_or_else(|| {
            "<owner backtrace unavailable; owner may have just unlocked>".to_owned()
        });
    let current = capture_backtrace();

    // eprintln! + exit(1) flushes stderr; abort() would not reliably do so.
    eprintln!(
        "Error: UncontendableLock locked more than once:\n\
         First by:\n\
         {owner}\n\
         Then by:\n\
         {current}"
    );
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn sequential_lock_unlock_smoke() {
        let lock = UncontendableLock::new();
        {
            let _g = lock.lock();
        }
        let _g = lock.lock();
    }

    #[test]
    fn default_matches_new() {
        let _ = UncontendableLock::default();
    }

    #[cfg(not(feature = "enabled"))]
    #[test]
    fn disabled_types_are_zero_sized() {
        assert_eq!(size_of::<UncontendableLock>(), 0);
        assert_eq!(size_of::<UncontendableGuard<'static>>(), 0);
    }

    #[cfg(not(feature = "enabled"))]
    #[test]
    fn disabled_allows_overlapping_guards() {
        // Production build: the lock is a pure annotation and must not abort.
        let lock = UncontendableLock::new();
        let _a = lock.lock();
        let _b = lock.lock();
    }

    #[cfg(feature = "enabled")]
    #[test]
    fn enabled_lock_is_not_zero_sized() {
        assert_ne!(size_of::<UncontendableLock>(), 0);
    }

    #[cfg(feature = "enabled")]
    #[test]
    fn trim_internal_frames_drops_crate_prefix_and_renumbers() {
        let raw = "\
   0: uncontendable_lock::capture_backtrace
             at ./src/lib.rs:1:1
   1: uncontendable_lock::UncontendableLock::lock
             at ./src/lib.rs:2:2
   2: app::holder
             at ./src/app.rs:3:3
   3: app::main
             at ./src/app.rs:4:4";
        let trimmed = trim_internal_frames(raw);
        assert!(
            !trimmed.contains("uncontendable_lock::"),
            "internal frames left in:\n{trimmed}"
        );
        assert!(
            trimmed.contains("   0: app::holder"),
            "expected renumbered caller frame:\n{trimmed}"
        );
        assert!(
            trimmed.contains("   1: app::main"),
            "expected second caller frame:\n{trimmed}"
        );
    }
}
