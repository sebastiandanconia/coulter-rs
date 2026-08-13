//! Subprocess tests for the abort-on-contention path.
//!
//! The library calls `process::exit(1)` on contention so a failed acquisition
//! cannot unwind into the caller. That would kill the test runner if exercised
//! in-process; these tests spawn the tiny bins under `src/bin/` instead.
//!
//! Gated on `feature = "enabled"`: without it the bins are not built
//! (`required-features` in Cargo.toml).

#![cfg(feature = "enabled")]

use std::process::Command;

fn run(bin_env_key: &str) -> std::process::Output {
    let path = std::env::var_os(bin_env_key).unwrap_or_else(|| {
        panic!(
            "{bin_env_key} not set; cargo should inject CARGO_BIN_EXE_* for [[bin]] targets"
        )
    });
    Command::new(&path)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn {path:?}: {e}"))
}

fn assert_contention_diagnostic(stderr: &str) {
    assert!(
        stderr.contains("UncontendableLock locked more than once"),
        "missing diagnostic banner in stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("First by:") && stderr.contains("Then by:"),
        "missing backtrace sections in stderr:\n{stderr}"
    );
    // Display-formatted backtraces are multi-line; Debug soup was the old bug.
    assert!(
        stderr.lines().count() > 6,
        "backtrace looks too short / not Display-formatted:\n{stderr}"
    );
}

#[test]
fn double_lock_aborts_with_diagnostic() {
    let output = run("CARGO_BIN_EXE_double_lock");

    assert!(
        !output.status.success(),
        "double_lock exited successfully; expected abort on contention"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "expected exit status 1, got {:?}",
        output.status
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_contention_diagnostic(&stderr);
}

#[test]
fn cross_thread_lock_aborts_with_diagnostic() {
    let output = run("CARGO_BIN_EXE_cross_thread_lock");

    assert!(
        !output.status.success(),
        "cross_thread_lock exited successfully; expected abort on contention"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "expected exit status 1, got {:?}",
        output.status
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_contention_diagnostic(&stderr);
}

#[test]
fn single_lock_succeeds() {
    let output = run("CARGO_BIN_EXE_single_lock");
    assert!(
        output.status.success(),
        "single_lock failed; stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
