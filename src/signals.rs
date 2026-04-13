//! Cross-platform Ctrl+C handling: request graceful save + exit from the main loop.

use std::sync::atomic::{AtomicBool, Ordering};

static INTERRUPT_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Register a handler that sets the interrupt flag (does not abort the process).
pub fn init() {
    let _ = ctrlc::set_handler(|| {
        INTERRUPT_REQUESTED.store(true, Ordering::SeqCst);
    });
}

/// Returns `true` once per interrupt edge (clears the flag).
pub fn take_interrupt() -> bool {
    INTERRUPT_REQUESTED.swap(false, Ordering::SeqCst)
}
