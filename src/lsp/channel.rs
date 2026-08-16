use std::{
    range::Range,
    path::Path,
    sync::Arc,
};

use expanda::declare_item;
use lsp_types::*;

use crate::{lang::Language, pos::Utf16Pos};

#[derive(Copy, Clone, Debug)]
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

#[declare_item(EDITOR_TO_LSP_MESSAGE)]
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

pub(crate) use EDITOR_TO_LSP_MESSAGE as editor_to_lsp_message_src;

pub type EditorToLspSender = tokio::sync::mpsc::UnboundedSender<EditorToLspMessage>;
pub type EditorToLspReceiver = tokio::sync::mpsc::UnboundedReceiver<EditorToLspMessage>;
pub type LspToEditorSender = std::sync::mpsc::Sender<LspToEditorMessage>;
pub type LspToEditorReceiver = std::sync::mpsc::Receiver<LspToEditorMessage>;

pub fn editor_to_lsp_channel() -> (EditorToLspSender, EditorToLspReceiver) {
    tokio::sync::mpsc::unbounded_channel()
}

pub fn lsp_to_editor_channel() -> (LspToEditorSender, LspToEditorReceiver) {
    std::sync::mpsc::channel()
}

pub struct LspChannels {
    pub incoming: EditorToLspReceiver,
    pub outgoing: LspToEditorSender,
}
