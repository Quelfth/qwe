use std::{
    collections::HashSet,
    env,
    future::Future,
    io,
    process::Stdio,
};

use async_lsp::{
    LanguageServer,
    ServerSocket,
    concurrency::ConcurrencyLayer,
    lsp_types::InitializeParams,
    panic::CatchUnwindLayer,
};
use async_process::Child;
use futures_lite::{AsyncBufReadExt as _, StreamExt as _};
use lsp_types::*;
use serde_json as json;
use tokio::{
    sync::mpsc::UnboundedReceiver,
    task::JoinHandle,
};
use tower::ServiceBuilder;

use crate::{
    aprintln::aprintln, log::log_msg, lsp::{client::{Client, ClientMessage}, log::LogLayer}
};

use channel::{
    LspChannels,
    LspToEditorSender,
};
use thread::lsp_thread;

pub mod channel;
mod thread;
mod client;
mod types;
mod server;
mod log;

pub use server::SpecialBehavior;

pub fn run_lsp_thread(channels: LspChannels) -> io::Result<std::thread::JoinHandle<()>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()?;
    let handle = std::thread::spawn(move || {
        let result = runtime.block_on(lsp_thread(channels));
        if let Err(e) = result {
            aprintln!("lsp errored: {e:?}");
        }
    });
    Ok(handle)
}
pub struct Server {
    join: JoinHandle<async_lsp::Result<()>>,
    socket: ServerSocket,
    caps: ServerCaps,
    client_channel: UnboundedReceiver<ClientMessage>,
    docs: HashSet<Url>,
    #[expect(unused)]
    tx: LspToEditorSender,
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
    fn spawn(command: &str, args: &[&str], tx: LspToEditorSender) -> async_lsp::Result<Self> {
        let (send, recv) = tokio::sync::mpsc::unbounded_channel();
        let (r#loop, socket) = async_lsp::MainLoop::new_client(|_| {
            ServiceBuilder::new()
                .layer(LogLayer)
                .layer(CatchUnwindLayer::default())
                .layer(ConcurrencyLayer::default())
                .service(Client { channel: send }.into_router())
        });

        let mut process = async_process::Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        let lsp_out = process.stdout.take().unwrap();
        let lsp_in = process.stdin.take().unwrap();
        let lsp_err = process.stderr.take().unwrap();

        let join = tokio::spawn(r#loop.run_buffered(lsp_out, lsp_in));
        tokio::spawn(async move {
            let mut lines = futures_lite::io::BufReader::new(lsp_err).lines();
            while let Some(line) = lines.next().await {
                if let Ok(msg) = line {
                    log_msg!(LspMessage, "{msg}");
                }
            }
        });

        Ok(Self {
            _process: process,
            join,
            caps: Default::default(),
            client_channel: recv,
            docs: Default::default(),
            socket,
            tx,
        })
    }

    pub async fn initialize(&mut self, options: Option<json::Value>) -> async_lsp::Result<InitializeResult> {
        self.socket
            .initialize(InitializeParams {
                workspace_folders: Some(vec![WorkspaceFolder {
                    uri: Url::from_file_path(env::current_dir()?).unwrap(),
                    name: "root".into(),
                }]),
                initialization_options: options,
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
                        diagnostic: Some(DiagnosticWorkspaceClientCapabilities {
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
                        code_action: Some(CodeActionClientCapabilities {
                            dynamic_registration: None,
                            code_action_literal_support: Some(CodeActionLiteralSupport {
                                code_action_kind: CodeActionKindLiteralSupport {
                                    value_set: [
                                        CodeActionKind::EMPTY,
                                        CodeActionKind::QUICKFIX,
                                        CodeActionKind::REFACTOR,
                                        CodeActionKind::SOURCE,
                                    ]
                                    .into_iter()
                                    .map(|k| k.as_str().to_owned())
                                    .collect(),
                                },
                            }),
                            is_preferred_support: Some(true),
                            ..Default::default()
                        }),
                        rename: Some(RenameClientCapabilities {
                            prepare_support: Some(true),
                            ..Default::default()
                        }),
                        diagnostic: Some(DiagnosticClientCapabilities {
                            dynamic_registration: Some(true),
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

    pub fn semantic_tokens(
        &mut self,
        doc_uri: Url,
    ) -> impl use<> + Future<Output = async_lsp::Result<Option<Vec<SemanticToken>>>> {
        let semtoks = self.caps.semtoks.then(||
            self
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
        );
        async {
            let Some(semtoks) = semtoks else {return Ok(None)};
            if let Some(semtoks) = semtoks.await? {
                let (SemanticTokensResult::Tokens(SemanticTokens { data, .. })
                | SemanticTokensResult::Partial(SemanticTokensPartialResult { data })) = semtoks;
                return Ok(Some(data));
            }
        
            Ok(None)
        }
    }
}
