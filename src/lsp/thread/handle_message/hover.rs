use crate::{pos::Utf16Pos, util::uri_from_path};

use super::prelude::*;

impl LspThread {
    pub async fn handle_hover(&mut self, lang: Language, path: Arc<Path>, pos: Utf16Pos) -> HandleResult {
        let Utf16Pos { line, column } = pos;
        if let Some(server) = self.servers.get_mut(&lang) {
            let Some(uri) = uri_from_path(&path) else {r#continue!()};
            if let Ok(Some(Hover { contents, .. })) = log_err!(server
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
                .await)
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
                self.tx.send(LspToEditorMessage::Hover { view })?;
            }
        }
        Ok(CONTINUE)
    }
}
