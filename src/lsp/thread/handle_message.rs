#![allow(unused)]

use std::{collections::hash_map, ops::ControlFlow, path::Path, sync::Arc};

use async_lsp::LanguageServer as _;
use expanda::expand;
use lsp_types::{DidOpenTextDocumentParams, TextDocumentContentChangeEvent, TextDocumentItem, Url};

use crate::{lang::{LangLspInfo, Language}, log::log_err, lsp::{
    Error, Server, channel::{EDITOR_TO_LSP_MESSAGE, EditorToLspMessage, GotoKind, LspToEditorMessage}, thread::LspThread
}, pos::Utf16Pos, util::uri_from_path};

type HandleResult = Result<ControlFlow<(), ()>, Error>;

const CONTINUE: ControlFlow<(), ()> = ControlFlow::Continue(());

macro r#continue() {
    return Ok(CONTINUE)
}

impl LspThread {
    pub async fn handle_message(&mut self, msg: EditorToLspMessage) -> HandleResult {
        expand! {
            <--use EDITOR_TO_LSP_MESSAGE
            <--let ($*^({$*.}). {$*($msgs*^(,). ,)}) = $EDITOR_TO_LSP_MESSAGE

            match msg {
                <--for $msg in $msgs {
                    <--match $msg {
                        ($msg.) => {
                            EditorToLspMessage::$msg => self.${(handle_) msg.snake_case}().await,
                        }
                        ($msg. {$*($arg. : $*^(,). ,)}) => {
                            EditorToLspMessage::$msg {
                                <--for $arg in $arg {
                                    $arg,
                                }
                            } => { self.${(handle_) msg.snake_case} (
                                <--for $arg in $arg {
                                    $arg,
                                }
                            ).await }
                        }
                    }
                }
            }
        }
    }

    async fn handle_open_doc(&mut self, lang: Language, path: Arc<Path>, text: String) -> HandleResult {
        let Some(LangLspInfo {
            id: lang_id,
            command,
            args,
            special_init,
            options,
        }) = lang.lsp_info()
        else {
            r#continue!()
        };
        if let hash_map::Entry::Vacant(e) = self.servers.entry(lang) {
            let Ok(mut server) = log_err!(Server::spawn(command, args, self.tx.clone())) else {r#continue!()};
            let Ok(init_result) = log_err!(server.initialize(options).await) else {r#continue!()};
            server.caps = (&init_result.capabilities).into();
            self.tx.send(LspToEditorMessage::NewLsp { lang, init_result })?;
            _= log_err!(server.initialized());
            server.special_init(special_init).await;
            e.insert(server);
        }
        let server = self.servers.get_mut(&lang).unwrap();
        let Some(doc_uri) = uri_from_path(&path) else {r#continue!()};
        _= log_err!(server.socket.did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: doc_uri.clone(),
                language_id: lang_id.to_owned(),
                version: 1,
                text,
            },
        }));
        if !server.docs.contains(&doc_uri) {
            server.docs.insert(doc_uri.clone());
        }
        server.refresh_semantic_tokens(doc_uri);

        Ok(CONTINUE)
    }

    async fn handle_change_doc(&mut self, lang: Language, path: Arc<Path>, changes: Vec<TextDocumentContentChangeEvent>, version: i32) -> HandleResult {todo!()}

    async fn handle_refresh_semantic_tokens(&mut self) -> HandleResult {todo!()}

    async fn handle_hover(&mut self, lang: Language, path: Arc<Path>, pos: Utf16Pos) -> HandleResult {todo!()}

    async fn handle_completion(&mut self, lang: Language, path: Arc<Path>, pos: Utf16Pos) -> HandleResult {todo!()}

    async fn handle_goto(&mut self, lang: Language, path: Arc<Path>, pos: Utf16Pos, kind: GotoKind) -> HandleResult {todo!()}

    async fn handle_code_actions(&mut self, lang: Language, path: Arc<Path>, pos: Utf16Pos) -> HandleResult {todo!()}

    async fn handle_rename(&mut self, lang: Language, path: Arc<Path>, pos: Utf16Pos) -> HandleResult {todo!()}

    async fn handle_complete_rename(&mut self, lang: Language, path: Arc<Path>, pos: Utf16Pos, name: String) -> HandleResult {todo!()}

    async fn handle_exit(&mut self) -> HandleResult {todo!()}

    async fn handle_save(&mut self, lang: Language, path: Arc<Path>) -> HandleResult {todo!()}
}
