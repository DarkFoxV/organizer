use crate::models::toast::{Toast, ToastKind};
use once_cell::sync::Lazy;
use std::sync::{
    RwLock,
    atomic::{AtomicU32, Ordering},
};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ToastService {
    pub id: u32,
    pub toast: Toast,
}

// Contador global de ID de toast
static NEXT_ID: AtomicU32 = AtomicU32::new(1);

// Pilha global protegida
static TOASTS: Lazy<RwLock<Vec<Toast>>> = Lazy::new(|| RwLock::new(Vec::new()));

/// Adiciona um novo toast no topo da pilha
fn push_toast(mut toast: Toast) {
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
    toast.id = Some(id);

    if let Ok(mut toasts) = TOASTS.write() {
        toasts.push(toast);
    }
}
/// Remove e retorna o toast mais recente (último inserido)
pub fn pop_toast() -> Option<Toast> {
    TOASTS.write().ok()?.pop()
}

pub fn push_success<S: Into<String>>(message: S) {
    let toast = Toast::new(ToastKind::Success, message.into(), Duration::from_secs(3));
    push_toast(toast);
}

pub fn push_error<E: Into<String>>(err: E) {
    let toast = Toast::new(ToastKind::Success, err.into(), Duration::from_secs(3));
    push_toast(toast);
}
