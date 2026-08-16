use std::ops::ControlFlow;

use expanda::{expand, using};

use crate::lsp::{
    channel::{
        EditorToLspMessage,
        editor_to_lsp_message_src as e2l_msg,
    },
    thread::LspThread,
};

impl LspThread {
    pub async fn handle_message(&mut self, msg: EditorToLspMessage) -> anyhow::Result<ControlFlow<(), ()>> {



        Ok(ControlFlow::Continue(()))
    }
}
