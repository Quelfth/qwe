use std::{
    path::Path,
    sync::{Arc, mpsc::Sender},
};

use lsp_types::{
    CompletionItem, Diagnostic, InitializeResult, Location, SemanticToken,
    TextDocumentContentChangeEvent, Url, CodeAction,
};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{lang::Language, pos::Utf16Pos};

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
}

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