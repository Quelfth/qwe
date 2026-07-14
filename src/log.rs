use std::fmt::{Debug, Display, Formatter, Result};

use append_only_vec::AppendOnlyVec;

use crate::lsp::channel::{EditorToLspMessage, LspToEditorMessage};

static LOG: AppendOnlyVec<LogEntry> = AppendOnlyVec::new();

pub macro log($log: expr) {{
    let log = &$log;
    LOG.push(LogEntry {
        category: Log::category(log),
        time: jiff::Zoned::now(),
        source: log_source!(),
        message: Log::message(log),
        details: Log::details(log),
    });
}}

pub macro log_err($result: expr) {
    $result.map_err(|e| log!(e))
}

pub macro log_msg($fmt: literal $(, $a: expr)* $(,)?) {
    log!(DisplayLog(format_args!($fmt $(, $a)*)))
}

pub fn log_iter() -> impl Iterator<Item = &'static LogEntry> {
    LOG.iter().rev()
}

#[derive(Clone, Debug)]
pub struct DebugLog<T>(pub T);

#[allow(unused)]
#[derive(Clone)]
pub struct DisplayLog<T>(pub T);

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum LogCategory {
    Debug,
    EditorToLspMessage,
    LspToEditorMessage,
    LspError,
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
            EditorToLspMessage::Exit => "exit".to_string(),
            EditorToLspMessage::Save { path, .. } => format!("save {path:?}"),
            #[allow(unused)]
            _ => "message type without logging implementation".to_string(),
        }
    }

    fn details(&self) -> String {
        format!("{self:?}")
    }
}

impl Log for LspToEditorMessage {
    fn category(&self) -> LogCategory { LogCategory::LspToEditorMessage }

    fn message(&self) -> String {
        match self {
            LspToEditorMessage::NewLsp { lang, .. } => format!("new lsp for {lang:?}"),
            LspToEditorMessage::SemanticTokens { uri, .. } => format!("semantic tokens for {uri}"),
            LspToEditorMessage::Diagnostics { uri, .. } => format!("diagnostics for {uri}"),
            LspToEditorMessage::Hover { .. } => "hover".to_string(),
            LspToEditorMessage::Completion { .. } => "completion".to_string(),
            LspToEditorMessage::Goto { .. } => "goto".to_string(),
            LspToEditorMessage::CodeActions { .. } => "code actions".to_string(),
            LspToEditorMessage::PrepareRename { text, .. } => format!("prepare rename from {text:?}"),
            LspToEditorMessage::Rename { .. } => "rename".to_string(),
        }
    }

    fn details(&self) -> String {
        String::new()
    }
}

impl Log for async_lsp::Error {
    fn category(&self) -> LogCategory { LogCategory::LspError }

    fn message(&self) -> String { self.to_string() }

    fn details(&self) -> String { String::new() }
}

impl<T: Debug> Log for DebugLog<T> {
    fn category(&self) -> LogCategory { LogCategory::Debug }

    fn message(&self) -> String {
        format!("{:?}", self.0)
    }

    fn details(&self) -> String { String::new() }
}

impl<T: Display> Log for DisplayLog<T> {
    fn category(&self) -> LogCategory { LogCategory::Debug }

    fn message(&self) -> String {
        format!("{}", self.0)
    }

    fn details(&self) -> String { String::new() }
}
