use crate::util::uri_from_path;

use super::prelude::*;

impl LspThread {
    pub async fn handle_save(&mut self, lang: Language, path: Arc<Path>) -> HandleResult {
        if let Some(server) = self.servers.get_mut(&lang) {
            let Some(uri) = uri_from_path(&path) else {r#continue!()};
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
        Ok(CONTINUE)
    }
}
