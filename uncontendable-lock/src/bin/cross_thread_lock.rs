//! Cross-thread contention. With `enabled`, must abort and print the
//! UncontendableLock diagnostic on stderr.

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use uncontendable_lock::UncontendableLock;

fn main() {
    let lock = Arc::new(UncontendableLock::new());
    let held = Arc::clone(&lock);

    let owner = thread::spawn(move || {
        let _guard = held.lock();
        // Keep the lock long enough for the main thread to contend.
        thread::sleep(Duration::from_millis(500));
    });

    // Ensure the owner thread has acquired before we contend.
    thread::sleep(Duration::from_millis(50));
    let _contender = lock.lock();

    // Unreachable when `enabled` works correctly.
    let _ = owner.join();
}
