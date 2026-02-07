use std::{
    boxed::Box,
    collections::{HashMap, VecDeque, hash_map::Entry},
    env,
    future::Future,
    io,
    marker::Send,
    ops::ControlFlow,
    pin::Pin,
    process::Stdio,
    result::Result,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use async_lsp::{
    LanguageClient, LanguageServer, ResponseError, ServerSocket, concurrency::ConcurrencyLayer,
    lsp_types::InitializeParams, panic::CatchUnwindLayer, router::Router, tracing::TracingLayer,
};
use async_process::Child;
use lsp_types::{
    ClientCapabilities, DidChangeTextDocumentParams, DidOpenTextDocumentParams, InitializeResult,
    InitializedParams, PublishDiagnosticsParams, SemanticToken, SemanticTokens,
    SemanticTokensClientCapabilities, SemanticTokensClientCapabilitiesRequests,
    SemanticTokensFullOptions, SemanticTokensParams, SemanticTokensPartialResult,
    SemanticTokensRegistrationOptions, SemanticTokensResult, SemanticTokensServerCapabilities,
    SemanticTokensWorkspaceClientCapabilities, ServerCapabilities, TextDocumentClientCapabilities,
    TextDocumentContentChangeEvent, TextDocumentItem, TokenFormat, Url,
    VersionedTextDocumentIdentifier, WorkspaceClientCapabilities, WorkspaceFolder,
};
use tokio::task::JoinHandle;
use tower::ServiceBuilder;
use tracing::Level;

use crate::{
    aprintln::aprintln,
    lang::LangLspInfo,
    lsp::channel::{EditorToLspMessage, LspToEditorMessage},
};

use channel::LspChannels;

pub mod channel;

pub fn run_lsp_thread(channels: LspChannels) -> io::Result<thread::JoinHandle<()>> {
    let runtime = tokio::runtime::Builder::new_current_thread().build()?;
    let handle = thread::spawn(move || {
        let result = runtime.block_on(lsp_thread(channels));
        if let Err(e) = result {
            aprintln!("{e:?}");
        }
    });
    Ok(handle)
}
struct Server {
    join: JoinHandle<async_lsp::Result<()>>,
    socket: ServerSocket,
    caps: ServerCaps,
    client_channel: Receiver<ClientMessage>,
    docs: Vec<Url>,
    _process: Child,
}

#[derive(Copy, Clone, Default)]
struct ServerCaps {
    semtoks: bool,
}

impl From<&ServerCapabilities> for ServerCaps {
    fn from(value: &ServerCapabilities) -> Self {
        Self {
            semtoks: value.semantic_tokens_provider.as_ref().is_some_and(|s| {
                let (SemanticTokensServerCapabilities::SemanticTokensOptions(o)
                | SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                    SemanticTokensRegistrationOptions {
                        semantic_tokens_options: o,
                        ..
                    },
                )) = s;
                o.full.as_ref().is_some_and(|o| {
                    matches!(
                        o,
                        SemanticTokensFullOptions::Bool(true)
                            | SemanticTokensFullOptions::Delta { delta: Some(_) }
                    )
                })
            }),
        }
    }
}

impl Server {
    fn spawn(command: &str) -> anyhow::Result<Self> {
        let (send, recv) = mpsc::channel();
        let (r#loop, socket) = async_lsp::MainLoop::new_client(|_| {
            ServiceBuilder::new()
                .layer(TracingLayer::default())
                .layer(CatchUnwindLayer::default())
                .layer(ConcurrencyLayer::default())
                .service(Client { channel: send }.into_router())
        });

        let mut process = async_process::Command::new(command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()?;

        let lsp_out = process.stdout.take().unwrap();
        let lsp_in = process.stdin.take().unwrap();

        let join = tokio::spawn(r#loop.run_buffered(lsp_out, lsp_in));

        Ok(Self {
            _process: process,
            join,
            caps: Default::default(),
            client_channel: recv,
            docs: Vec::new(),
            socket,
        })
    }

    pub async fn initialize(&mut self) -> async_lsp::Result<InitializeResult> {
        self.socket
            .initialize(InitializeParams {
                workspace_folders: Some(vec![WorkspaceFolder {
                    uri: Url::from_file_path(env::current_dir()?).unwrap(),
                    name: "root".into(),
                }]),
                capabilities: ClientCapabilities {
                    workspace: Some(WorkspaceClientCapabilities {
                        semantic_tokens: Some(SemanticTokensWorkspaceClientCapabilities {
                            refresh_support: Some(true),
                        }),
                        ..Default::default()
                    }),
                    text_document: Some(TextDocumentClientCapabilities {
                        semantic_tokens: Some(SemanticTokensClientCapabilities {
                            dynamic_registration: Some(false),
                            requests: SemanticTokensClientCapabilitiesRequests {
                                range: Some(false),
                                full: Some(SemanticTokensFullOptions::Bool(true)),
                            },
                            token_types: vec![],
                            token_modifiers: vec![],
                            formats: vec![],
                            overlapping_token_support: Some(true),
                            multiline_token_support: Some(true),
                            server_cancel_support: Some(false),
                            augments_syntax_tokens: Some(true),
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            })
            .await
    }

    pub fn initialized(&mut self) -> async_lsp::Result<()> {
        self.socket.initialized(InitializedParams {})
    }

    pub async fn semantic_tokens(
        &mut self,
        doc_uri: Url,
    ) -> async_lsp::Result<Option<Vec<SemanticToken>>> {
        if self.caps.semtoks {
            let semtoks = self
                .socket
                .semantic_tokens_full(SemanticTokensParams {
                    work_done_progress_params: lsp_types::WorkDoneProgressParams {
                        work_done_token: None,
                    },
                    partial_result_params: lsp_types::PartialResultParams {
                        partial_result_token: None,
                    },
                    text_document: lsp_types::TextDocumentIdentifier { uri: doc_uri },
                })
                .await?;
            if let Some(semtoks) = semtoks {
                match semtoks {
                    SemanticTokensResult::Tokens(_) => aprintln!("full"),
                    SemanticTokensResult::Partial(_) => aprintln!("partial"),
                }
                let (SemanticTokensResult::Tokens(SemanticTokens { data, .. })
                | SemanticTokensResult::Partial(SemanticTokensPartialResult { data })) = semtoks;
                return Ok(Some(data));
            }
        }

        Ok(None)
    }
}

pub async fn lsp_thread(channels: LspChannels) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_ansi(false)
        .with_writer(io::stderr)
        .init();

    let mut servers = HashMap::new();
    let mut init_delay_queue = VecDeque::new();

    loop {
        if let Ok(msg) = channels.incoming.recv_timeout(Duration::from_millis(20)) {
            match msg {
                EditorToLspMessage::OpenDoc { lang, path, text } => {
                    let Some(LangLspInfo {
                        id: lang_id,
                        command,
                    }) = lang.lsp_info()
                    else {
                        continue;
                    };
                    if let Entry::Vacant(e) = servers.entry(lang) {
                        let mut server = Server::spawn(command)?;
                        let init_result = server.initialize().await?;
                        server.caps = (&init_result.capabilities).into();
                        channels
                            .outgoing
                            .send(LspToEditorMessage::NewLsp { lang, init_result })?;
                        server.initialized()?;
                        e.insert(server);
                    }
                    let server = servers.get_mut(&lang).unwrap();
                    let doc_uri = Url::from_file_path(path.canonicalize().unwrap()).unwrap();
                    server.socket.did_open(DidOpenTextDocumentParams {
                        text_document: TextDocumentItem {
                            uri: doc_uri.clone(),
                            language_id: lang_id.to_owned(),
                            version: 0,
                            text,
                        },
                    })?;
                    server.docs.push(doc_uri.clone());
                    if let Some(tokens) = server.semantic_tokens(doc_uri.clone()).await? {
                        channels
                            .outgoing
                            .send(LspToEditorMessage::SemanticTokens { tokens })?;
                    }
                    init_delay_queue.push_back((lang, doc_uri, Instant::now()))
                }
                EditorToLspMessage::Exit => break,
                EditorToLspMessage::RefreshSemanticTokens => {
                    for server in servers.values_mut() {
                        aprintln!("manual refresh");
                        for doc in server.docs.clone() {
                            let Some(semtoks) = server.semantic_tokens(doc).await? else {
                                continue;
                            };

                            channels
                                .outgoing
                                .send(LspToEditorMessage::SemanticTokens { tokens: semtoks })?;
                        }
                    }
                }
                EditorToLspMessage::ChangeDoc {
                    lang,
                    path,
                    changes,
                    version,
                } => {
                    if let Some(server) = servers.get_mut(&lang) {
                        let uri = Url::from_file_path(path.canonicalize()?).unwrap();
                        server.socket.did_change(DidChangeTextDocumentParams {
                            text_document: VersionedTextDocumentIdentifier {
                                uri: uri.clone(),
                                version,
                            },
                            content_changes: changes,
                        })?;
                        if let Some(semtoks) = server.semantic_tokens(uri).await? {
                            channels
                                .outgoing
                                .send(LspToEditorMessage::SemanticTokens { tokens: semtoks })?;
                        }
                    }
                }
            }
        }
        for server in servers.values_mut() {
            if let Ok(msg) = server
                .client_channel
                .recv_timeout(Duration::from_millis(20))
            {
                match msg {
                    ClientMessage::SemanticTokensRefresh => {
                        aprintln!("do the refresh");
                        for doc in server.docs.clone() {
                            let Some(semtoks) = server.semantic_tokens(doc).await? else {
                                continue;
                            };

                            channels
                                .outgoing
                                .send(LspToEditorMessage::SemanticTokens { tokens: semtoks })?;
                        }
                    }
                }
            }
        }
        const INIT_DELAY: Duration = Duration::from_millis(500);
        if init_delay_queue
            .front()
            .is_some_and(|f| f.2.elapsed() > INIT_DELAY)
        {
            let Some((lang, uri, _)) = init_delay_queue.pop_front() else {
                unreachable!()
            };

            let server = servers.get_mut(&lang).unwrap();

            if let Some(tokens) = server.semantic_tokens(uri).await? {
                channels
                    .outgoing
                    .send(LspToEditorMessage::SemanticTokens { tokens })?;
            }
        }
    }

    for server in servers.into_values() {
        server.join.await??;
    }

    Ok(())
}

enum ClientMessage {
    SemanticTokensRefresh,
}

pub struct Client {
    channel: Sender<ClientMessage>,
}

impl Client {
    fn into_router(self) -> Router<Self> {
        let mut router = Router::from_language_client(self);
        router.event(Self::on_stop);
        router
    }

    fn on_stop(&mut self, _: Stop) -> ControlFlow<async_lsp::Result<()>> {
        ControlFlow::Break(Ok(()))
    }
}

pub struct Stop;

impl LanguageClient for Client {
    type Error = ResponseError;
    type NotifyResult = ControlFlow<async_lsp::Result<()>>;

    fn publish_diagnostics(&mut self, _: PublishDiagnosticsParams) -> Self::NotifyResult {
        ControlFlow::Continue(())
    }

    fn semantic_tokens_refresh(
        &mut self,
        (): (),
    ) -> Pin<Box<dyn Future<Output = Result<(), Self::Error>> + Send + 'static>> {
        aprintln!("semantic tokens refresh");
        let channel = self.channel.clone();
        Box::pin(async move {
            channel.send(ClientMessage::SemanticTokensRefresh).unwrap();
            Ok(())
        })
    }
}
