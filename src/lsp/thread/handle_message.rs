#![allow(unused)]

use std::ops::ControlFlow;


use expanda::{expand, using};

use crate::lsp::{
    channel::{
        EditorToLspMessage,
    },
    thread::LspThread,
};

impl LspThread {
    pub async fn handle_message(&mut self, msg: EditorToLspMessage) -> anyhow::Result<ControlFlow<(), ()>> {



        Ok(ControlFlow::Continue(()))
    }
}
