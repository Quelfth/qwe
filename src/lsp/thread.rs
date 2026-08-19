
use std::{
    collections::{HashMap, hash_map::Entry},
    time::Duration,
};

use async_lsp::LanguageServer as _;
use lsp_types::*;
use tokio::time::timeout;
//use tracing::Level;

use crate::{
    lang::{LangLspInfo, Language},
    log::{log, log_err},
    pos::Utf16Pos,
};

use super::{
    channel::{
        self,
        EditorToLspMessage,
        EditorToLspReceiver,
        LspChannels,
        LspToEditorMessage,
        LspToEditorSender,
    },
    ClientMessage,
    Server,
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

pub async fn lsp_thread(channels: LspChannels) -> anyhow::Result<()> {
    //tracing_subscriber::fmt()
    //    .with_max_level(Level::DEBUG)
    //    .with_ansi(false)
    //    .with_writer(io::stderr)
    //    .init();
    
    let mut cx = LspThread::new(channels);

    fn refresh_semantic_tokens(server: &mut Server, doc: Url, tx: LspToEditorSender) {
        let future = server.semantic_tokens(doc.clone());
        tokio::spawn(async move {
            let Ok(Some(semtoks)) = future.await.map_err(|e| log!(e)) else { return };
            tx.send(LspToEditorMessage::SemanticTokens {
                uri: doc.clone(),
                tokens: semtoks,
            }).unwrap();
        });
    }

    loop {
        if let Ok(Some(msg)) = timeout(Duration::from_millis(20), cx.rx.recv()).await {
            log!(msg);
            match msg {
                EditorToLspMessage::OpenDoc { lang, path, text } => {
                    let Some(LangLspInfo {
                        id: lang_id,
                        command,
                        args,
                        special_init,
                        options,
                    }) = lang.lsp_info()
                    else {
                        continue;
                    };
                    if let Entry::Vacant(e) = cx.servers.entry(lang) {
                        let Ok(mut server) = log_err!(Server::spawn(command, args, cx.tx.clone())) else {continue};
                        let Ok(init_result) = log_err!(server.initialize(options).await) else {continue};
                        server.caps = (&init_result.capabilities).into();
                        cx.tx.send(LspToEditorMessage::NewLsp { lang, init_result })?;
                        _=log_err!(server.initialized());
                        server.special_init(special_init).await;
                        e.insert(server);
                    }
                    let server = cx.servers.get_mut(&lang).unwrap();
                    let doc_uri = Url::from_file_path(path.canonicalize().unwrap()).unwrap();
                    server.socket.did_open(DidOpenTextDocumentParams {
                        text_document: TextDocumentItem {
                            uri: doc_uri.clone(),
                            language_id: lang_id.to_owned(),
                            version: 1,
                            text,
                        },
                    })?;
                    if !server.docs.contains(&doc_uri) {
                        server.docs.insert(doc_uri.clone());
                    }
                    refresh_semantic_tokens(server, doc_uri, cx.tx.clone());
                }
                EditorToLspMessage::Exit => break,
                EditorToLspMessage::RefreshSemanticTokens => {
                    for server in cx.servers.values_mut() {
                        for doc in server.docs.clone() {
                            refresh_semantic_tokens(server, doc, cx.tx.clone());
                        }
                    }
                }
                EditorToLspMessage::Hover {
                    lang,
                    path,
                    pos: Utf16Pos { line, column },
                } => {
                    if let Some(server) = cx.servers.get_mut(&lang) {
                        let uri = Url::from_file_path(path.canonicalize().unwrap_or((*path).to_owned())).unwrap();
                        if let Ok(Some(Hover { contents, .. })) = log_err!(server
                            .socket
                            .hover(HoverParams {
                                text_document_position_params: TextDocumentPositionParams {
                                    text_document: TextDocumentIdentifier { uri },
                                    position: Position {
                                        line: line.inner() as _,
                                        character: column.inner() as _,
                                    },
                                },
                                work_done_progress_params: WorkDoneProgressParams {
                                    work_done_token: None,
                                },
                            })
                            .await)
                        {
                            let view = match contents {
                                HoverContents::Scalar(string) => match string {
                                    MarkedString::String(string) => string,
                                    MarkedString::LanguageString(LanguageString {
                                        value, ..
                                    }) => value,
                                },
                                HoverContents::Array(marked_strings) => marked_strings
                                    .into_iter()
                                    .map(|s| match s {
                                        MarkedString::String(string) => string,
                                        MarkedString::LanguageString(LanguageString {
                                            value,
                                            ..
                                        }) => value + "\n===\n",
                                    })
                                    .collect(),
                                HoverContents::Markup(MarkupContent { value, .. }) => value,
                            };
                            cx.tx.send(LspToEditorMessage::Hover { view })?;
                        }
                    }
                }
                EditorToLspMessage::Completion {
                    lang,
                    path,
                    pos: Utf16Pos { line, column },
                } => {
                    if let Some(server) = cx.servers.get_mut(&lang) {
                        let uri = Url::from_file_path(path.canonicalize()?).unwrap();
                        if let Ok(Some(response)) = log_err!(server
                            .socket
                            .completion(CompletionParams {
                                text_document_position: TextDocumentPositionParams {
                                    text_document: TextDocumentIdentifier { uri },
                                    position: Position {
                                        line: line.inner() as _,
                                        character: column.inner() as _,
                                    },
                                },
                                work_done_progress_params: Default::default(),
                                partial_result_params: Default::default(),
                                context: None,
                            })
                            .await)
                        {
                            let items = match response {
                                CompletionResponse::Array(items) => items,
                                CompletionResponse::List(CompletionList { items, .. }) => items,
                            };
                            cx.tx.send(LspToEditorMessage::Completion { items })?;
                        }
                    }
                }
                EditorToLspMessage::Goto {
                    lang,
                    path,
                    pos: Utf16Pos { line, column },
                    kind,
                } => {
                    if let Some(server) = cx.servers.get_mut(&lang) {
                        let uri = Url::from_file_path(path.canonicalize()?).unwrap();
                        use channel::GotoKind::*;
                        let text_document_position_params = TextDocumentPositionParams {
                            text_document: TextDocumentIdentifier { uri },
                            position: Position {
                                line: line.inner() as _,
                                character: column.inner() as _,
                            },
                        };
                        let params = GotoDefinitionParams {
                            text_document_position_params: text_document_position_params.clone(),
                            work_done_progress_params: Default::default(),
                            partial_result_params: Default::default(),
                        };
                        fn locs(goto: Option<GotoDefinitionResponse>) -> Option<Vec<Location>> {
                            Some(match goto? {
                                GotoDefinitionResponse::Scalar(location) => vec![location],
                                GotoDefinitionResponse::Array(locations) => locations,
                                GotoDefinitionResponse::Link(_) => todo!(),
                            })
                        }
                        if let Some(locations) = match kind {
                            Definition => locs(log_err!(server.socket.definition(params).await).ok().flatten()),
                            Declaration => locs(log_err!(server.socket.declaration(params).await).ok().flatten()),
                            Implementation => locs(log_err!(server.socket.implementation(params).await).ok().flatten()),
                            TypeDefinition => locs(log_err!(server.socket.type_definition(params).await).ok().flatten()),
                            References => {
                                log_err!(server
                                    .socket
                                    .references(ReferenceParams {
                                        text_document_position: text_document_position_params,
                                        work_done_progress_params: Default::default(),
                                        partial_result_params: Default::default(),
                                        context: ReferenceContext {
                                            include_declaration: true,
                                        },
                                    })
                                    .await).ok().flatten()
                            }
                        } {
                            cx.tx.send(LspToEditorMessage::Goto { locations })?;
                        }
                    }
                }
                EditorToLspMessage::CodeActions {
                    lang,
                    path,
                    pos: Utf16Pos { line, column },
                } => {
                    if let Some(server) = cx.servers.get_mut(&lang) {
                        let uri = Url::from_file_path(path.canonicalize()?).unwrap();
                        let pos = Position {
                            line: line.inner() as _,
                            character: column.inner() as _,
                        };
                        if let Ok(Some(response)) = log_err!(server
                            .socket
                            .code_action(CodeActionParams {
                                text_document: TextDocumentIdentifier { uri },
                                range: Range {
                                    start: pos,
                                    end: pos,
                                },
                                context: CodeActionContext {
                                    diagnostics: vec![],
                                    only: None,
                                    trigger_kind: Some(lsp_types::CodeActionTriggerKind::INVOKED),
                                },
                                partial_result_params: Default::default(),
                                work_done_progress_params: Default::default(),
                            })
                            .await)
                        {
                            let mut actions = Vec::new();
                            for action in response {
                                use lsp_types::CodeActionOrCommand;
                                match action {
                                    CodeActionOrCommand::CodeAction(action) => {
                                        actions.push(action);
                                    }
                                    CodeActionOrCommand::Command(_) => {
                                        todo!()
                                    }
                                }
                            }
                            cx.tx.send(LspToEditorMessage::CodeActions { actions })?;
                        }
                    }
                }
                EditorToLspMessage::Rename {
                    lang,
                    path,
                    pos: Utf16Pos { line, column },
                } => {
                    if let Some(server) = cx.servers.get_mut(&lang) {
                        let uri = Url::from_file_path(path.canonicalize()?).unwrap();
                        if let Ok(Some(response)) = log_err!(server.socket.prepare_rename(TextDocumentPositionParams {
                            text_document: TextDocumentIdentifier { uri },
                            position: Position {
                                line: line.inner() as _,
                                character: column.inner() as _,
                            },
                        }).await) {
                            let (range, text) = match response {
                                PrepareRenameResponse::Range(range) => (Some(range), None),
                                PrepareRenameResponse::RangeWithPlaceholder { range, placeholder } => (Some(range), Some(placeholder)),
                                PrepareRenameResponse::DefaultBehavior { .. } => (None, None),
                            };

                            let range = range.map(|Range { start, end }| Utf16Pos::from_lsp_pos(start)..Utf16Pos::from_lsp_pos(end));

                            cx.tx.send(LspToEditorMessage::PrepareRename { range, text })?;
                        }
                    }
                }
                EditorToLspMessage::CompleteRename { lang, path, pos: Utf16Pos { line, column }, name } => {
                     if let Some(server) = cx.servers.get_mut(&lang) {
                        let uri = Url::from_file_path(path.canonicalize()?).unwrap();
                        if let Ok(Some(edit)) = log_err!(server.socket.rename(RenameParams{ 
                            text_document_position: TextDocumentPositionParams {
                                text_document: TextDocumentIdentifier { uri },
                                position: Position {
                                    line: line.inner() as _,
                                    character: column.inner() as _,
                                },
                            },
                            new_name: name,
                            work_done_progress_params: Default::default(),
                        }).await) {
                            cx.tx.send(LspToEditorMessage::Rename { edit })?;
                        }
                    }
                },
                EditorToLspMessage::ChangeDoc {
                    lang,
                    path,
                    changes,
                    version,
                } => {
                    if let Some(server) = cx.servers.get_mut(&lang) {
                        let uri = Url::from_file_path(path.canonicalize()?).unwrap();
                        _=log_err!(server.socket.did_change(DidChangeTextDocumentParams {
                            text_document: VersionedTextDocumentIdentifier {
                                uri: uri.clone(),
                                version,
                            },
                            content_changes: changes,
                        }));
                        refresh_semantic_tokens(server, uri, cx.tx.clone());
                    }
                }
                EditorToLspMessage::Save { lang, path } => {
                    if let Some(server) = cx.servers.get_mut(&lang) {
                        let uri = Url::from_file_path(path.canonicalize()?).unwrap();
                        _=log_err!(server.socket.did_save(DidSaveTextDocumentParams {
                            text_document: TextDocumentIdentifier { uri: uri.clone() },
                            text: None,
                        }));
                        _=log_err!(server
                            .socket
                            .did_change_watched_files(DidChangeWatchedFilesParams {
                                changes: vec![FileEvent {
                                    uri,
                                    typ: FileChangeType::CHANGED,
                                }],
                            }));
                    }
                }
            }
        }
        for server in cx.servers.values_mut() {
            if let Ok(Some(msg)) =
                timeout(Duration::from_millis(20), server.client_channel.recv()).await
            {
                match msg {
                    ClientMessage::SemanticTokensRefresh => {
                        for doc in server.docs.clone() {
                            refresh_semantic_tokens(server, doc.clone(), cx.tx.clone());
                        }
                    }
                    ClientMessage::PublishDiagnostics { uri, diagnostics } => {
                        server.docs.insert(uri.clone());
                        cx.tx.send(LspToEditorMessage::Diagnostics { uri, diagnostics })?;
                    }
                }
            }
        }
    }

    for server in cx.servers.into_values() {
        server.join.await??;
    }

    Ok(())
}
