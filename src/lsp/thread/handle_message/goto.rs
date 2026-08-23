use crate::{lsp::channel::GotoKind, pos::Utf16Pos, util::uri_from_path};

use super::prelude::*;

impl LspThread {
    pub async fn handle_goto(&mut self, lang: Language, path: Arc<Path>, pos: Utf16Pos, kind: GotoKind) -> HandleResult {
        let Utf16Pos { line, column } = pos;
        if let Some(server) = self.servers.get_mut(&lang) {
            let Some(uri) = uri_from_path(&path) else {r#continue!()};
            use GotoKind::*;
            let text_document_position_params = TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position {
                    line: line.inner() as _,
                    character: column.inner() as _,
                },
            };
            let params = GotoDefinitionParams {
                text_document_position_params: text_document_position_params.clone(),
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
            };
            fn locs(goto: Option<GotoDefinitionResponse>) -> Option<Vec<Location>> {
                Some(match goto? {
                    GotoDefinitionResponse::Scalar(location) => vec![location],
                    GotoDefinitionResponse::Array(locations) => locations,
                    GotoDefinitionResponse::Link(_) => todo!(),
                })
            }
            if let Some(locations) = match kind {
                Definition => locs(log_err!(server.socket.definition(params).await).ok().flatten()),
                Declaration => locs(log_err!(server.socket.declaration(params).await).ok().flatten()),
                Implementation => locs(log_err!(server.socket.implementation(params).await).ok().flatten()),
                TypeDefinition => locs(log_err!(server.socket.type_definition(params).await).ok().flatten()),
                References => {
                    log_err!(server
                        .socket
                        .references(ReferenceParams {
                            text_document_position: text_document_position_params,
                            work_done_progress_params: Default::default(),
                            partial_result_params: Default::default(),
                            context: ReferenceContext {
                                include_declaration: true,
                            },
                        })
                        .await).ok().flatten()
                }
            } {
                self.tx.send(LspToEditorMessage::Goto { locations })?;
            }
        }
        Ok(CONTINUE)
    }
}
