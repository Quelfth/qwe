use crate::{pos::Utf16Pos, util::uri_from_path};

use super::prelude::*;

impl LspThread {
    pub async fn handle_rename(&mut self, lang: Language, path: Arc<Path>, pos: Utf16Pos) -> HandleResult {
        let Utf16Pos { line, column } = pos;
        if let Some(server) = self.servers.get_mut(&lang) {
            let Some(uri) = uri_from_path(&path) else {r#continue!()};
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

                self.tx.send(LspToEditorMessage::PrepareRename { range, text })?;
            }
        }
        Ok(CONTINUE)
    }

    pub async fn handle_complete_rename(&mut self, lang: Language, path: Arc<Path>, pos: Utf16Pos, name: String) -> HandleResult {
        let Utf16Pos { line, column } = pos;
        if let Some(server) = self.servers.get_mut(&lang) {
            let Some(uri) = uri_from_path(&path) else {r#continue!()};
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
                self.tx.send(LspToEditorMessage::Rename { edit })?;
            }
        }
        Ok(CONTINUE)
    }
}
