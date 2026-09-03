use async_lsp::{AnyRequest, LspService};
use tower::{Layer, Service};

use crate::log::{DisplayLog, log, log_msg, LogCategory};

mod pretty_json;
pub use pretty_json::pretty_json;

#[derive(Clone, Default)]
pub struct LogLayer;

pub struct LogMiddleware<S> {
    inner: S
}

impl<S> Layer<S> for LogLayer {
    type Service = LogMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LogMiddleware { inner }
    }
}

impl<S: LspService> LspService for LogMiddleware<S> {
    fn notify(&mut self, notif: async_lsp::AnyNotification) -> std::ops::ControlFlow<async_lsp::Result<()>> {
        log!(DisplayLog {
            category: LogCategory::LspNotification,
            message: &notif.method,
            details: pretty_json(&notif.params),
        });
        self.inner.notify(notif)
    }

    fn emit(&mut self, event: async_lsp::AnyEvent) -> std::ops::ControlFlow<async_lsp::Result<()>> {
        log_msg!(LspEvent, "{}", event.type_name());
        self.inner.emit(event)
    }
}

impl<S: Service<AnyRequest>> Service<AnyRequest> for LogMiddleware<S> {
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: AnyRequest) -> Self::Future {
        log!(DisplayLog {
            category: LogCategory::LspRequest,
            message: &req.method,
            details: pretty_json(&req.params),
        });
        self.inner.call(req)
    }
}
