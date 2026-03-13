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
    thread,
    time::{Duration, Instant},
};

use async_lsp::{
    LanguageClient, LanguageServer, ResponseError, ServerSocket, concurrency::ConcurrencyLayer,
    lsp_types::InitializeParams, panic::CatchUnwindLayer, router::Router, tracing::TracingLayer,
};
use async_process::Child;
use lsp_types::{
    ClientCapabilities, CompletionClientCapabilities, CompletionItemCapability, CompletionItemKind,
    CompletionItemKindCapability, CompletionList, CompletionParams, CompletionResponse,
    ConfigurationParams, Diagnostic, DiagnosticTag, DidChangeTextDocumentParams,
    DidChangeWatchedFilesClientCapabilities, DidChangeWatchedFilesParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, FileChangeType, FileEvent,
    GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverClientCapabilities, HoverContents,
    HoverParams, InitializeResult, InitializedParams, LanguageString, Location, LogMessageParams,
    LogTraceParams, MarkedString, MarkupContent, MarkupKind, PartialResultParams, Position,
    ProgressParams, PublishDiagnosticsClientCapabilities, PublishDiagnosticsParams,
    ReferenceContext, ReferenceParams, SemanticToken, SemanticTokens,
    SemanticTokensClientCapabilities, SemanticTokensClientCapabilitiesRequests,
    SemanticTokensFullOptions, SemanticTokensParams, SemanticTokensPartialResult,
    SemanticTokensRegistrationOptions, SemanticTokensResult, SemanticTokensServerCapabilities,
    SemanticTokensWorkspaceClientCapabilities, ServerCapabilities, ShowDocumentParams,
    ShowDocumentResult, TagSupport, TextDocumentClientCapabilities, TextDocumentIdentifier,
    TextDocumentItem, TextDocumentPositionParams, TextDocumentSyncClientCapabilities, Url,
    VersionedTextDocumentIdentifier, WindowClientCapabilities, WorkDoneProgressCreateParams,
    WorkDoneProgressParams, WorkspaceClientCapabilities, WorkspaceFolder,
};
use serde_json as json;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
    time::timeout,
};
use tower::ServiceBuilder;
use tracing::{Level, info};

use crate::{
    aprintln::aprintln,
    lang::LangLspInfo,
    lsp::channel::{EditorToLspMessage, LspToEditorMessage},
    pos::Utf16Pos,
};

use channel::LspChannels;

pub mod channel;

pub fn run_lsp_thread(channels: LspChannels) -> io::Result<thread::JoinHandle<()>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()?;
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
    client_channel: UnboundedReceiver<ClientMessage>,
    docs: Vec<Url>,
    _process: Child,
}

#[derive(Clone, Default)]
struct ServerCaps {
    semtoks: bool,
}

impl From<&ServerCapabilities> for ServerCaps {
    fn from(value: &ServerCapabilities) -> Self {
        //aprintln!("Lsp Capabilities:\n{:#?}", value);
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
        let (send, recv) = tokio::sync::mpsc::unbounded_channel();
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
                        configuration: Some(true),
                        did_change_watched_files: Some(DidChangeWatchedFilesClientCapabilities {
                            dynamic_registration: None,
                            relative_pattern_support: None,
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
                        publish_diagnostics: Some(PublishDiagnosticsClientCapabilities {
                            related_information: Some(true),
                            version_support: Some(true),
                            tag_support: Some(TagSupport {
                                value_set: vec![
                                    DiagnosticTag::UNNECESSARY,
                                    DiagnosticTag::DEPRECATED,
                                ],
                            }),
                            code_description_support: Some(true),
                            ..Default::default()
                        }),
                        synchronization: Some(TextDocumentSyncClientCapabilities {
                            did_save: Some(true),
                            ..Default::default()
                        }),
                        hover: Some(HoverClientCapabilities {
                            content_format: Some(vec![MarkupKind::Markdown]),
                            ..Default::default()
                        }),
                        completion: Some(CompletionClientCapabilities {
                            completion_item: Some(CompletionItemCapability {
                                documentation_format: Some(vec![MarkupKind::Markdown]),
                                ..Default::default()
                            }),
                            completion_item_kind: Some(CompletionItemKindCapability {
                                value_set: Some(vec![
                                    CompletionItemKind::METHOD,
                                    CompletionItemKind::FUNCTION,
                                    CompletionItemKind::CONSTRUCTOR,
                                    CompletionItemKind::FIELD,
                                    CompletionItemKind::VARIABLE,
                                    CompletionItemKind::CLASS,
                                    CompletionItemKind::INTERFACE,
                                    CompletionItemKind::MODULE,
                                    CompletionItemKind::PROPERTY,
                                    CompletionItemKind::ENUM,
                                    CompletionItemKind::KEYWORD,
                                    CompletionItemKind::SNIPPET,
                                    CompletionItemKind::FILE,
                                    CompletionItemKind::FOLDER,
                                    CompletionItemKind::ENUM_MEMBER,
                                    CompletionItemKind::CONSTANT,
                                    CompletionItemKind::STRUCT,
                                    CompletionItemKind::OPERATOR,
                                    CompletionItemKind::TYPE_PARAMETER,
                                ]),
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    window: Some(WindowClientCapabilities {
                        work_done_progress: Some(true),
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
                    work_done_progress_params: WorkDoneProgressParams {
                        work_done_token: None,
                    },
                    partial_result_params: PartialResultParams {
                        partial_result_token: None,
                    },
                    text_document: TextDocumentIdentifier { uri: doc_uri },
                })
                .await?;
            if let Some(semtoks) = semtoks {
                let (SemanticTokensResult::Tokens(SemanticTokens { data, .. })
                | SemanticTokensResult::Partial(SemanticTokensPartialResult { data })) = semtoks;
                return Ok(Some(data));
            }
        }

        Ok(None)
    }
}

pub async fn lsp_thread(mut channels: LspChannels) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_ansi(false)
        .with_writer(io::stderr)
        .init();
    info!("wow!");

    let mut servers = HashMap::new();
    let mut init_delay_queue = VecDeque::new();

    loop {
        if let Ok(Some(msg)) = timeout(Duration::from_millis(20), channels.incoming.recv()).await {
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
                            version: 1,
                            text,
                        },
                    })?;
                    server.docs.push(doc_uri.clone());
                    if let Some(tokens) = server.semantic_tokens(doc_uri.clone()).await? {
                        channels
                            .outgoing
                            .send(LspToEditorMessage::SemanticTokens { uri: doc_uri.clone(), tokens })?;
                    }

                    init_delay_queue.push_back((lang, doc_uri, Instant::now()))
                }
                EditorToLspMessage::Exit => break,
                EditorToLspMessage::RefreshSemanticTokens => {
                    for server in servers.values_mut() {
                        for doc in server.docs.clone() {
                            if let Some(semtoks) = server.semantic_tokens(doc.clone()).await? {
                                channels
                                    .outgoing
                                    .send(LspToEditorMessage::SemanticTokens { uri: doc.clone(), tokens: semtoks })?;
                            }
                        }
                    }
                }
                EditorToLspMessage::Hover {
                    lang,
                    path,
                    pos: Utf16Pos { line, column },
                } => {
                    if let Some(server) = servers.get_mut(&lang) {
                        let uri = Url::from_file_path(path.canonicalize()?).unwrap();
                        if let Some(Hover { contents, .. }) = server
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
                            .await?
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
                            channels.outgoing.send(LspToEditorMessage::Hover { view })?;
                        }
                    }
                }
                EditorToLspMessage::Completion {
                    lang,
                    path,
                    pos: Utf16Pos { line, column },
                } => {
                    if let Some(server) = servers.get_mut(&lang) {
                        let uri = Url::from_file_path(path.canonicalize()?).unwrap();
                        if let Some(response) = server
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
                            .await?
                        {
                            let items = match response {
                                CompletionResponse::Array(items) => items,
                                CompletionResponse::List(CompletionList { items, .. }) => items,
                            };
                            channels
                                .outgoing
                                .send(LspToEditorMessage::Completion { items })?;
                        }
                    }
                }
                EditorToLspMessage::Goto {
                    lang,
                    path,
                    pos: Utf16Pos { line, column },
                    kind,
                } => {
                    if let Some(server) = servers.get_mut(&lang) {
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
                            Definition => locs(server.socket.definition(params).await?),
                            Declaration => locs(server.socket.declaration(params).await?),
                            Implementation => locs(server.socket.implementation(params).await?),
                            TypeDefinition => locs(server.socket.type_definition(params).await?),
                            References => {
                                server
                                    .socket
                                    .references(ReferenceParams {
                                        text_document_position: text_document_position_params,
                                        work_done_progress_params: Default::default(),
                                        partial_result_params: Default::default(),
                                        context: ReferenceContext {
                                            include_declaration: true,
                                        },
                                    })
                                    .await?
                            }
                        } {
                            channels
                                .outgoing
                                .send(LspToEditorMessage::Goto { locations })?;
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
                        if let Some(semtoks) = server.semantic_tokens(uri.clone()).await? {
                            channels
                                .outgoing
                                .send(LspToEditorMessage::SemanticTokens { uri, tokens: semtoks })?;
                        }
                    }
                }
                EditorToLspMessage::Save { lang, path } => {
                    if let Some(server) = servers.get_mut(&lang) {
                        let uri = Url::from_file_path(path.canonicalize()?).unwrap();
                        server.socket.did_save(DidSaveTextDocumentParams {
                            text_document: TextDocumentIdentifier { uri: uri.clone() },
                            text: None,
                        })?;
                        server
                            .socket
                            .did_change_watched_files(DidChangeWatchedFilesParams {
                                changes: vec![FileEvent {
                                    uri,
                                    typ: FileChangeType::CHANGED,
                                }],
                            })?;
                    }
                }
            }
        }
        for server in servers.values_mut() {
            if let Ok(Some(msg)) =
                timeout(Duration::from_millis(20), server.client_channel.recv()).await
            {
                match msg {
                    ClientMessage::SemanticTokensRefresh => {
                        for doc in server.docs.clone() {
                            let Some(semtoks) = server.semantic_tokens(doc.clone()).await? else {
                                continue;
                            };

                            channels
                                .outgoing
                                .send(LspToEditorMessage::SemanticTokens { uri: doc, tokens: semtoks })?;
                        }
                    }
                    ClientMessage::PublishDiagnostics { uri, diagnostics } => {
                        if server.docs.contains(&uri) {
                            channels
                                .outgoing
                                .send(LspToEditorMessage::Diagnostics { uri, diagnostics })?;
                        }
                    }
                }
            }
        }
    }

    for server in servers.into_values() {
        server.join.await??;
    }

    Ok(())
}

enum ClientMessage {
    PublishDiagnostics {
        uri: Url,
        diagnostics: Vec<Diagnostic>,
    },
    SemanticTokensRefresh,
}

pub struct Client {
    channel: UnboundedSender<ClientMessage>,
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

type Response<T> = Pin<Box<dyn Future<Output = Result<T, ResponseError>> + Send + 'static>>;

impl LanguageClient for Client {
    type Error = ResponseError;
    type NotifyResult = ControlFlow<async_lsp::Result<()>>;

    fn publish_diagnostics(&mut self, params: PublishDiagnosticsParams) -> Self::NotifyResult {
        let PublishDiagnosticsParams {
            uri, diagnostics, ..
        } = params;
        _ = self
            .channel
            .send(ClientMessage::PublishDiagnostics { uri, diagnostics });

        ControlFlow::Continue(())
    }

    fn work_done_progress_create(&mut self, _: WorkDoneProgressCreateParams) -> Response<()> {
        Box::pin(async move { Ok(()) })
    }

    fn progress(&mut self, _: ProgressParams) -> Self::NotifyResult {
        ControlFlow::Continue(())
    }

    fn semantic_tokens_refresh(&mut self, (): ()) -> Response<()> {
        let channel = self.channel.clone();
        Box::pin(async move {
            channel.send(ClientMessage::SemanticTokensRefresh).unwrap();
            Ok(())
        })
    }

    fn configuration(&mut self, _: ConfigurationParams) -> Response<Vec<json::Value>> {
        Box::pin(async { Ok(vec![]) })
    }

    fn workspace_folders(&mut self, (): ()) -> Response<Option<Vec<WorkspaceFolder>>> {
        Box::pin(async { Ok(None) })
    }

    fn show_document(&mut self, _: ShowDocumentParams) -> Response<ShowDocumentResult> {
        Box::pin(async { Ok(ShowDocumentResult { success: false }) })
    }

    fn log_message(&mut self, _: LogMessageParams) -> Self::NotifyResult {
        ControlFlow::Continue(())
    }

    fn log_trace(&mut self, _: LogTraceParams) -> Self::NotifyResult {
        ControlFlow::Continue(())
    }
}
