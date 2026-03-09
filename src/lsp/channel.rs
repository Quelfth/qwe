use std::{
    path::Path,
    sync::{Arc, mpsc::Sender},
};

use lsp_types::{Diagnostic, InitializeResult, SemanticToken, TextDocumentContentChangeEvent, Url};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::lang::Language;

pub enum LspToEditorMessage {
    NewLsp {
        lang: Language,
        init_result: InitializeResult,
    },
    SemanticTokens {
        tokens: Vec<SemanticToken>,
    },
    Diagnostics {
        uri: Url,
        diagnostics: Vec<Diagnostic>,
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
    Save {
        lang: Language,
        path: Arc<Path>,
    },
}

pub struct LspChannels {
    pub incoming: UnboundedReceiver<EditorToLspMessage>,
    pub outgoing: Sender<LspToEditorMessage>,
}
