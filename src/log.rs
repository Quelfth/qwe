use std::time::Instant;

static LOG: boxcar::Vec<LogEntry> = boxcar::Vec::new();

pub macro log([$cat: ident]$fmt: literal$({$($vars:tt)*})?) {
    LOG.push(LogEntry {
        category: LogCategory::$cat,
        time: Instant::now(),
        source: log_source!(),
        message: format!($fmt$(, $($vars)*)?),
        details: String::new(),
    })
}


pub enum LogCategory {
    LspMessage,
}

pub struct LogSource {
    file: &'static str,
    line: u32,
    column: u32,
}

macro log_source() {
    LogSource {
        file: file!(),
        line: line!(),
        column: column!(),
    }
}

pub struct LogEntry {
    category: LogCategory,
    time: Instant,
    source: LogSource,
    message: String,
    details: String,
}