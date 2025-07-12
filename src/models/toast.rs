use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum ToastKind {
    Success,
    Error,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub id: Option<u32>,
    pub message: String,
    pub kind: ToastKind,
    pub created: Instant,
    pub duration: Duration,
}

impl Toast {
    pub fn new(kind: ToastKind, message: String, duration: Duration) -> Toast {
        Toast {
            id: None,
            message,
            kind,
            created: Instant::now(),
            duration,
        }
    }
}