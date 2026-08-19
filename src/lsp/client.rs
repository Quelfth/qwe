use std::{ops::ControlFlow, pin::Pin};

use async_lsp::{LanguageClient, ResponseError, router::Router};
use lsp_types::*;
use serde_json as json;
use tokio::sync::mpsc::UnboundedSender;

use crate::log::log_msg;

pub enum ClientMessage {
    PublishDiagnostics {
        uri: Url,
        diagnostics: Vec<Diagnostic>,
    },
    SemanticTokensRefresh,
}

pub struct Client {
    pub channel: UnboundedSender<ClientMessage>,
}

impl Client {
    pub fn into_router(self) -> Router<Self> {
        let mut router = Router::from_language_client(self);
        router.event(Self::on_stop);
        router.unhandled_notification(|_, notification| {
            let msg = notification.method.as_str();
            log_msg!("Unhandled Notification: {msg}");
            ControlFlow::Continue(())
        });
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

    fn show_message(&mut self, _: ShowMessageParams) -> Self::NotifyResult {
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

    fn register_capability(&mut self, _: RegistrationParams) -> Response<()> {
        log_msg!("register capability");
        Box::pin(async { Ok(()) })
    }

    fn unregister_capability(&mut self, _: UnregistrationParams) -> Response<()>{
        log_msg!("unregister capability");
        Box::pin(async { Ok(()) })
    }

    fn inline_value_refresh(&mut self, _: ()) -> Response<()>{
        log_msg!("inline value refresh");
        Box::pin(async { Ok(()) })
    }

    fn inlay_hint_refresh(&mut self, _ : ()) -> Response<()>{
        log_msg!("inlay hint refresh");
        Box::pin(async { Ok(()) })
    }

    fn workspace_diagnostic_refresh(&mut self, _: ()) -> Response<()>{
        log_msg!("workspace diagnostic refresh");
        Box::pin(async { Ok(()) })
    }

    fn show_message_request(&mut self, _: ShowMessageRequestParams) -> Response<Option<MessageActionItem>>{
        log_msg!("show message request");
        Box::pin(async { Ok(None) })
    }

    fn code_lens_refresh(&mut self, _: ()) -> Response<()>{
        log_msg!("code lens refresh");
        Box::pin(async { Ok(()) })
    }

    fn apply_edit(&mut self, _: ApplyWorkspaceEditParams) -> Response<ApplyWorkspaceEditResponse>{
        log_msg!("apply edit");
        Box::pin(async { Ok(ApplyWorkspaceEditResponse {
            applied: false,
            failure_reason: None,
            failed_change: None
        }) })
    }

    fn telemetry_event(&mut self, _: OneOf<json::Map<String, json::Value>, Vec<json::Value>>) -> Self::NotifyResult {
        ControlFlow::Continue(())
    }

    fn cancel_request(&mut self, _: CancelParams) -> Self::NotifyResult {
        ControlFlow::Continue(())
    }
}
