//! Same-thread re-entrant acquisition. With `enabled`, must abort and print
//! the UncontendableLock diagnostic on stderr.

use uncontendable_lock::UncontendableLock;

fn main() {
    let lock = UncontendableLock::new();
    let _first = lock.lock();
    // Second acquisition must fail: std::sync::Mutex is not re-entrant, and
    // the design claims no second holder of any kind.
    let _second = lock.lock();
}
