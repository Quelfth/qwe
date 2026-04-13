use std::{
    ops::Range, path::Path, sync::{Arc, mpsc::Sender}
};

use lsp_types::{
    CodeAction, CompletionItem, Diagnostic, InitializeResult, Location, SemanticToken, TextDocumentContentChangeEvent, Url, WorkspaceEdit
};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{lang::Language, log::{Log, LogCategory}, pos::Utf16Pos};

#[derive(Debug)]
pub enum GotoKind {
    Definition,
    Declaration,
    Implementation,
    References,
    TypeDefinition,
}

pub enum LspToEditorMessage {
    NewLsp {
        lang: Language,
        init_result: InitializeResult,
    },
    SemanticTokens {
        uri: Url,
        tokens: Vec<SemanticToken>,
    },
    Diagnostics {
        uri: Url,
        diagnostics: Vec<Diagnostic>,
    },
    Hover {
        view: String,
    },
    Completion {
        items: Vec<CompletionItem>,
    },
    Goto {
        locations: Vec<Location>,
    },
    CodeActions {
        actions: Vec<CodeAction>,
    },
    PrepareRename {
        range: Option<Range<Utf16Pos>>,
        text: Option<String>,
    },
    Rename {
        edit: WorkspaceEdit,
    },
}

#[derive(Debug)]
pub enum EditorToLspMessage {
    OpenDoc {
        lang: Language,
        path: Arc<Path>,
        text: String,
    },
    ChangeDoc {
        lang: Language,
        path: Arc<Path>,
        changes: Vec<TextDocumentContentChangeEvent>,
        version: i32,
    },
    RefreshSemanticTokens,
    Hover {
        lang: Language,
        path: Arc<Path>,
        pos: Utf16Pos,
    },
    Completion {
        lang: Language,
        path: Arc<Path>,
        pos: Utf16Pos,
    },
    Goto {
        lang: Language,
        path: Arc<Path>,
        pos: Utf16Pos,
        kind: GotoKind,
    },
    CodeActions {
        lang: Language,
        path: Arc<Path>,
        pos: Utf16Pos,
    },
    Rename {
        lang: Language,
        path: Arc<Path>,
        pos: Utf16Pos,
    },
    CompleteRename {
        lang: Language,
        path: Arc<Path>,
        pos: Utf16Pos,
        name: String,
    },
    Exit,
    Save {
        lang: Language,
        path: Arc<Path>,
    },
}

pub struct LspChannels {
    pub incoming: UnboundedReceiver<EditorToLspMessage>,
    pub outgoing: Sender<LspToEditorMessage>,
}