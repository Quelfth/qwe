use std::fmt::{Display, Formatter, Result};

use append_only_vec::AppendOnlyVec;

use crate::lsp::channel::{EditorToLspMessage, LspToEditorMessage};

static LOG: AppendOnlyVec<LogEntry> = AppendOnlyVec::new();

pub macro log($log: expr) {
    let log = &$log;
    LOG.push(LogEntry {
        category: Log::category(log),
        time: jiff::Zoned::now(),
        source: log_source!(),
        message: Log::message(log),
        details: Log::details(log),
    })
}

pub fn log_iter() -> impl Iterator<Item = &'static LogEntry> {
    LOG.iter().rev()
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum LogCategory {
    EditorToLspMessage,
    LspToEditorMessage,
}

#[derive(Debug)]
pub struct LogSource {
    file: &'static str,
    line: u32,
    column: u32,
}

impl Display for LogSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

macro log_source() {
    LogSource {
        file: file!(),
        line: line!(),
        column: column!(),
    }
}

pub struct LogEntry {
    pub category: LogCategory,
    pub time: jiff::Zoned,
    pub source: LogSource,
    pub message: String,
    #[expect(unused)]
    pub details: String,
}

pub trait Log {
    fn category(&self) -> LogCategory;
    fn message(&self) -> String;
    fn details(&self) -> String;
}

impl Log for EditorToLspMessage {
    fn category(&self) -> LogCategory {
        LogCategory::EditorToLspMessage
    }

    fn message(&self) -> String {
        match self {
            EditorToLspMessage::OpenDoc { path, .. } => format!("open document {path:?}"),
            EditorToLspMessage::ChangeDoc { path, version, .. } => {
                format!("change document {path:?} (version {version:?})")
            }
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
            _ => format!("message type without logging implementation"),
        }
    }

    fn details(&self) -> String {
        match self {
            _ => format!("{self:?}"),
        }
    }
}

impl Log for LspToEditorMessage {
    fn category(&self) -> LogCategory { LogCategory::LspToEditorMessage }

    fn message(&self) -> String {
        match self {
            LspToEditorMessage::NewLsp { lang, .. } => format!("new lsp for {lang:?}"),
            LspToEditorMessage::SemanticTokens { uri, .. } => format!("semantic tokens for {uri}"),
            LspToEditorMessage::Diagnostics { uri, .. } => format!("diagnostics for {uri}"),
            LspToEditorMessage::Hover { .. } => format!("hover"),
            LspToEditorMessage::Completion { .. } => format!("completion"),
            LspToEditorMessage::Goto { .. } => format!("goto"),
            LspToEditorMessage::CodeActions { .. } => format!("code actions"),
            LspToEditorMessage::PrepareRename { text, .. } => format!("prepare rename from {text:?}"),
            LspToEditorMessage::Rename { .. } => format!("rename"),
        }
    }

    fn details(&self) -> String {
        String::new()
    }
}
