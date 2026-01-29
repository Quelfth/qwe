use std::{fmt::Write, sync::LazyLock};

use mutx::Mutex;

macro_rules! aprintln {
    ($($t:tt)*) => {$crate::aprintln::__aprintln(format_args!{$($t)*})};
}
pub(crate) use aprintln;
#[allow(unused)]
macro_rules! aprint {
    ($($t:tt)*) => {$crate::aprintln::__aprint(format_args!{$($t)*})};
}
#[allow(unused)]
pub(crate) use aprint;

static PRINT_STREAM: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::from("".to_owned()));
pub fn __aprint(args: std::fmt::Arguments<'_>) {
    let mut stream = PRINT_STREAM.lock();
    stream.write_fmt(args).unwrap();
}

pub fn __aprintln(args: std::fmt::Arguments<'_>) {
    let mut stream = PRINT_STREAM.lock();
    stream.write_fmt(args).unwrap();
    stream.write_char('\n').unwrap();
}

pub fn print_print_stream() {
    print!("{}", PRINT_STREAM.lock())
}
