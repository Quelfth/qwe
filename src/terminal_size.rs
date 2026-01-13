use std::sync::atomic::{AtomicU32, Ordering};

static TERMINAL_SIZE: AtomicU32 = AtomicU32::new(0);

pub fn set_terminal_size(width: u16, height: u16) -> bool {
    let value = (width as u32) << 16 | height as u32;
    let different = TERMINAL_SIZE.load(Ordering::Relaxed) != value;
    TERMINAL_SIZE.store(value, Ordering::Relaxed);
    different
}

pub fn terminal_size() -> (u16, u16) {
    let value = TERMINAL_SIZE.load(Ordering::Relaxed);
    ((value >> 16) as u16, value as u16)
}
