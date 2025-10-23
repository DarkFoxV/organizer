use crate::models::toast::{Toast, ToastKind};
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tokio::sync::mpsc;

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

static TOAST_CHANNEL: Lazy<(mpsc::UnboundedSender<Toast>, std::sync::Mutex<Option<mpsc::UnboundedReceiver<Toast>>>)> = Lazy::new(|| {
    let (tx, rx) = mpsc::unbounded_channel();
    (tx, std::sync::Mutex::new(Some(rx)))
});

pub fn take_toast_receiver() -> Option<mpsc::UnboundedReceiver<Toast>> {
    TOAST_CHANNEL.1.lock().ok()?.take()
}

fn push_toast(mut toast: Toast) {
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    toast.id = Some(id);

    let _ = TOAST_CHANNEL.0.send(toast);
}

pub fn push_success<S: Into<String>>(message: S) {
    let toast = Toast::new(ToastKind::Success, message.into(), Duration::from_secs(3));
    push_toast(toast);
}

pub fn push_error<E: Into<String>>(err: E) {
    let toast = Toast::new(ToastKind::Error, err.into(), Duration::from_secs(3));
    push_toast(toast);
}