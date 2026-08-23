use std::ops::ControlFlow;

use expanda::expand;

use crate::{
    lsp::{
        Error,
        channel::{EDITOR_TO_LSP_MESSAGE, EditorToLspMessage},
        thread::LspThread,
    },
};

mod change_doc;
mod code_actions;
mod completion;
mod goto;
mod hover;
mod open_doc;
mod rename;
mod save;

mod prelude {
    pub(super) use {
        std::{path::Path, sync::Arc},
        async_lsp::LanguageServer as _,
        lsp_types::*,
        crate::{
            lang::Language,
            log::log_err,
            lsp::{
                channel::LspToEditorMessage,
                thread::LspThread,
            },
        },
        super::{CONTINUE, r#continue, HandleResult},
    };
}

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

    async fn handle_exit(&mut self) -> HandleResult { Ok(ControlFlow::Break(())) }

    async fn handle_refresh_semantic_tokens(&mut self) -> HandleResult {
        for server in self.servers.values_mut() {
            for doc in server.docs.clone() {
                server.refresh_semantic_tokens(doc);
            }
        }
        Ok(CONTINUE)
    }
}
