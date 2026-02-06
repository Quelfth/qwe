use std::{
    path::Path,
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
};

use lsp_types::{InitializeResult, SemanticToken};

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
    RefreshSemanticTokens,
    Exit,
}

pub struct LspChannels {
    pub incoming: Receiver<EditorToLspMessage>,
    pub outgoing: Sender<LspToEditorMessage>,
}
