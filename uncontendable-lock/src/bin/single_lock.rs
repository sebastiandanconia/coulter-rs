//! Acquires the lock exactly once. Must exit cleanly with `enabled`.

use uncontendable_lock::UncontendableLock;

fn main() {
    let lock = UncontendableLock::new();
    let _guard = lock.lock();
}
