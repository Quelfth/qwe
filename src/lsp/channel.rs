use std::{
    path::Path,
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
};

use lsp_types::{Diagnostic, InitializeResult, SemanticToken, TextDocumentContentChangeEvent, Url};

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
    pub incoming: Receiver<EditorToLspMessage>,
    pub outgoing: Sender<LspToEditorMessage>,
}
