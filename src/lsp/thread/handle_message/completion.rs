use crate::{pos::Utf16Pos, util::uri_from_path};

use super::prelude::*;

impl LspThread {
    pub async fn handle_completion(&mut self, lang: Language, path: Arc<Path>, pos: Utf16Pos) -> HandleResult {
    let Utf16Pos { line, column } = pos;
    if let Some(server) = self.servers.get_mut(&lang) {
        let Some(uri) = uri_from_path(&path) else {r#continue!()};
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
            self.tx.send(LspToEditorMessage::Completion { items })?;
        }
    }
    Ok(CONTINUE)
}
}
