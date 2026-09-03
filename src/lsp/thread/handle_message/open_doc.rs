use std::collections::hash_map;

use crate::{lang::LangLspInfo, lsp::Server, util::uri_from_path};

use super::prelude::*;

impl LspThread {
    pub async fn handle_open_doc(&mut self, lang: Language, path: Arc<Path>, text: String) -> HandleResult {
        let Some(LangLspInfo {
            id: lang_id,
            command,
            args,
            special_init,
            options,
            severity_map: _,
        }) = lang.lsp_info()
        else {
            r#continue!()
        };
        if let hash_map::Entry::Vacant(e) = self.servers.entry(lang) {
            let Ok(mut server) = log_err!(Server::spawn(command, args, self.tx.clone())) else {r#continue!()};
            let Ok(init_result) = log_err!(server.initialize(options).await) else {r#continue!()};
            server.caps = (&init_result.capabilities).into();
            self.tx.send(LspToEditorMessage::NewLsp { lang, init_result })?;
            _= log_err!(server.initialized());
            server.special_init(special_init).await;
            e.insert(server);
        }
        let server = self.servers.get_mut(&lang).unwrap();
        let Some(doc_uri) = uri_from_path(&path) else {r#continue!()};
        _= log_err!(server.socket.did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: doc_uri.clone(),
                language_id: lang_id.to_owned(),
                version: 1,
                text,
            },
        }));
        if !server.docs.contains(&doc_uri) {
            server.docs.insert(doc_uri.clone());
        }
        server.refresh_semantic_tokens(doc_uri.clone());
        server.refresh_diagnostics(doc_uri);

        Ok(CONTINUE)
    }
}