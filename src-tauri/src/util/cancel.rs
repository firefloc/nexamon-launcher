use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tokio::sync::oneshot;

/// Global cancellation flag, checked between steps.
static CANCELLED: AtomicBool = AtomicBool::new(false);
/// Kill signal sender — triggers child process kill in the select! loop.
static KILL_TX: Mutex<Option<oneshot::Sender<()>>> = Mutex::new(None);

/// Request cancellation: sets flag and sends kill signal to child process.
pub fn request_cancel() {
    log::info!("[cancel] Cancellation requested");
    CANCELLED.store(true, Ordering::SeqCst);
    if let Ok(mut guard) = KILL_TX.lock() {
        if let Some(tx) = guard.take() {
            log::info!("[cancel] Sending kill signal to child process");
            let _ = tx.send(());
        }
    }
}

/// Check if cancellation was requested. Returns Err if cancelled.
pub fn check_cancelled() -> Result<(), String> {
    if CANCELLED.load(Ordering::SeqCst) {
        Err("Operation cancelled".into())
    } else {
        Ok(())
    }
}

/// Reset the cancellation flag. Call at the start of a new operation.
pub fn reset() {
    CANCELLED.store(false, Ordering::SeqCst);
    if let Ok(mut guard) = KILL_TX.lock() {
        *guard = None;
    }
}

/// Create a kill channel. Returns the receiver to use in select!.
/// The sender is stored globally for request_cancel() to use.
pub fn kill_channel() -> oneshot::Receiver<()> {
    let (tx, rx) = oneshot::channel();
    if let Ok(mut guard) = KILL_TX.lock() {
        *guard = Some(tx);
    }
    rx
}
