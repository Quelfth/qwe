use std::{
    io::{self, stdout},
    sync::atomic::{AtomicBool, Ordering},
};

use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::aprintln::print_print_stream;

static IS_SETUP: AtomicBool = AtomicBool::new(false);
fn is_setup() -> bool {
    IS_SETUP.load(Ordering::Relaxed)
}

pub fn setup() -> io::Result<()> {
    if is_setup() {
        return Ok(());
    }
    terminal::enable_raw_mode()?;
    execute! {
        stdout(),
        EnterAlternateScreen,
        EnableMouseCapture,
        cursor::Hide,
    }?;
    IS_SETUP.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn teardown() -> io::Result<()> {
    if !is_setup() {
        return Ok(());
    }
    execute! {
        stdout(),
        cursor::Show,
        DisableMouseCapture,
        LeaveAlternateScreen,
    }?;
    terminal::disable_raw_mode()?;
    IS_SETUP.store(false, Ordering::Relaxed);
    print_print_stream();
    Ok(())
}
