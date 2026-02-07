use std::{
    path::Path,
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
};

use lsp_types::{
    DidChangeTextDocumentParams, InitializeResult, SemanticToken, TextDocumentContentChangeEvent,
};

use crate::lang::Language;

pub enum LspToEditorMessage {
    NewLsp {
        lang: Language,
        init_result: InitializeResult,
    },
    SemanticTokens {
        tokens: Vec<SemanticToken>,
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
    Exit,
}

pub struct LspChannels {
    pub incoming: Receiver<EditorToLspMessage>,
    pub outgoing: Sender<LspToEditorMessage>,
}
