use std::{fmt::Write, sync::LazyLock};

use mutx::Mutex;

pub macro aprintln($($t:tt)*) {
    __aprintln(format_args!{$($t)*})
}

#[allow(unused)]
pub macro aprint($($t:tt)*) {
    __aprint(format_args!{$($t)*})
}

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
