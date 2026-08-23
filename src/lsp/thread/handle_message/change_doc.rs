use crate::util::uri_from_path;

use super::prelude::*;

impl LspThread {
    pub async fn handle_change_doc(&mut self, lang: Language, path: Arc<Path>, changes: Vec<TextDocumentContentChangeEvent>, version: i32) -> HandleResult {
        if let Some(server) = self.servers.get_mut(&lang) {
            let Some(uri) = uri_from_path(&path) else {r#continue!()};
            _=log_err!(server.socket.did_change(DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier {
                    uri: uri.clone(),
                    version,
                },
                content_changes: changes,
            }));
            server.refresh_semantic_tokens(uri);
        }
        Ok(CONTINUE)
    }
}
