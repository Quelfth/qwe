
use std::{
    collections::HashMap,
    ops::ControlFlow::*,
    sync::mpsc::SendError,
    time::Duration,
};

use lsp_types::Registration;
use tokio::time::timeout;

use crate::{
    lang::Language,
    log::{DisplayLog, LogCategory, log, log_err, log_msg}, lsp::server::DiagnosticHandler,
};

use super::{
    channel::{
        EditorToLspReceiver,
        LspChannels,
        LspToEditorMessage,
        LspToEditorSender,
    },
    ClientMessage,
    Server,
    log::pretty_json,
};

mod handle_message;

struct LspThread {
    rx: EditorToLspReceiver,
    tx: LspToEditorSender,
    servers: HashMap<Language, Server>,
}

impl LspThread {
    fn new(channels: LspChannels) -> Self {
        let LspChannels { incoming, outgoing } = channels;
        Self {
            rx: incoming,
            tx: outgoing,
            servers: Default::default(),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Send(#[allow(unused)] Box<SendError<LspToEditorMessage>>),
}

impl From<SendError<LspToEditorMessage>> for Error {
    fn from(value: SendError<LspToEditorMessage>) -> Self {
        Self::Send(Box::new(value))
    }
}

pub async fn lsp_thread(channels: LspChannels) -> Result<(), Error> {
    let mut cx = LspThread::new(channels);

    loop {
        if let Ok(Some(msg)) = timeout(Duration::from_millis(20), cx.rx.recv()).await {
            log!(msg);
            match cx.handle_message(msg).await? {
                Continue(()) => (),
                Break(()) => break,
            }
        }
        for server in cx.servers.values_mut() {
            if let Ok(Some(msg)) =
                timeout(Duration::from_millis(20), server.client_channel.recv()).await
            {
                match msg {
                    ClientMessage::SemanticTokensRefresh => {
                        for doc in server.docs.clone() {
                            server.refresh_semantic_tokens(doc.clone());
                        }
                    }
                    ClientMessage::PublishDiagnostics { uri, diagnostics } => {
                        server.docs.insert(uri.clone());
                        cx.tx.send(LspToEditorMessage::Diagnostics { uri, diagnostics })?;
                    }
                    ClientMessage::RegisterCapability(Registration { id, method, register_options }) => {
                        match &*method {
                            "textDocument/diagnostic" => {try {
                                let options = register_options.as_ref()?.as_object()?;
                                server.diagnostic_handlers.push(DiagnosticHandler {
                                    registration_id: Some(id.clone()),
                                    identifier: options.get("identifier")?.as_str()?.to_owned(),
                                    inter_file_dependencies: options.get("interFileDependencies").and_then(|o| o.as_bool()).unwrap_or_default(),
                                    workspace: options.get("workspaceDiagnostics").and_then(|o| o.as_bool()).unwrap_or_default(),
                                });
                            };},
                            _ => log_msg!("ignored registration of unsupported method {method}"),
                        }
                        log!(DisplayLog {
                            category: LogCategory::Debug,
                            message: "register lsp capability",
                            details: format!("{id}\n{method}\n{}", register_options.map(|o| pretty_json(&o)).unwrap_or_default()),
                        })
                    },
                }
            }
        }
    }

    for server in cx.servers.into_values() {
        let Ok(j) = server.join.await else {continue};
        _= log_err!(j);
    }

    Ok(())
}
