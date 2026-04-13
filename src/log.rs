use std::time::Instant;

use crate::lsp::channel::EditorToLspMessage;

static LOG: boxcar::Vec<LogEntry> = boxcar::Vec::new();

pub macro log($log: expr) {
    let log = &$log;
    LOG.push(LogEntry {
        category: Log::category(log),
        time: Instant::now(),
        source: log_source!(),
        message: Log::message(log),
        details: Log::details(log),
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

pub trait Log {
    fn category(&self) -> LogCategory;
    fn message(&self) -> String;
    fn details(&self) -> String;
}

impl Log for EditorToLspMessage {
    fn category(&self) -> LogCategory { LogCategory::LspMessage }

    fn message(&self) -> String {
        match self {
            EditorToLspMessage::OpenDoc { path, .. } => format!("open document {path:?}"),
            EditorToLspMessage::ChangeDoc { path, version, .. } => format!("change document {path:?} (version {version:?})"),
            EditorToLspMessage::RefreshSemanticTokens => "refresh semantic tokens".to_owned(),
            EditorToLspMessage::Hover { pos, .. } => format!("hover at {pos:?}"),
            EditorToLspMessage::Completion { pos, .. } => format!("completion at {pos:?}"),
            EditorToLspMessage::Goto { pos, kind, .. } => format!("goto {kind:?} from {pos:?}"),
            EditorToLspMessage::CodeActions { pos, .. } => format!("code actions at {pos:?}"),
            EditorToLspMessage::Rename { pos, .. } => format!("rename at {pos:?}"),
            EditorToLspMessage::CompleteRename { name, .. } => format!("rename to {name:?}"),
            EditorToLspMessage::Exit => format!("exit"),
            EditorToLspMessage::Save { path, .. } => format!("save {path:?}"),
            #[allow(unused)]
            _ => format!("message type without logging implementation")
        }
    }

    fn details(&self) -> String {
        match self {
            _ => format!("{self:?}"),
        }
    }
}