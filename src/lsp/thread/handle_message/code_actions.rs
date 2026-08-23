use crate::{pos::Utf16Pos, util::uri_from_path};

use super::prelude::*;

impl LspThread {
    pub async fn handle_code_actions(&mut self, lang: Language, path: Arc<Path>, pos: Utf16Pos) -> HandleResult {
        let Utf16Pos { line, column } = pos;
        if let Some(server) = self.servers.get_mut(&lang) {
            let Some(uri) = uri_from_path(&path) else {r#continue!()};
            let pos = Position {
                line: line.inner() as _,
                character: column.inner() as _,
            };
            if let Ok(Some(response)) = log_err!(server
                .socket
                .code_action(CodeActionParams {
                    text_document: TextDocumentIdentifier { uri },
                    range: Range {
                        start: pos,
                        end: pos,
                    },
                    context: CodeActionContext {
                        diagnostics: vec![],
                        only: None,
                        trigger_kind: Some(lsp_types::CodeActionTriggerKind::INVOKED),
                    },
                    partial_result_params: Default::default(),
                    work_done_progress_params: Default::default(),
                })
                .await)
            {
                let mut actions = Vec::new();
                for action in response {
                    use lsp_types::CodeActionOrCommand;
                    match action {
                        CodeActionOrCommand::CodeAction(action) => {
                            actions.push(action);
                        }
                        CodeActionOrCommand::Command(_) => {
                            todo!()
                        }
                    }
                }
                self.tx.send(LspToEditorMessage::CodeActions { actions })?;
            }
        }
        Ok(CONTINUE)
    }
}
