use std::io;

use crate::{
    document::diagnostics::{Diagnostic, Severity},
    editor::{Editor, completer::Completer, markdown_view::MarkdownGadget},
    language_server::LanguageServer,
    lsp::channel::{EditorToLspMessage, LspToEditorMessage},
    pos::Utf16Pos,
    range_sequence::RangeSequence,
};

impl Editor {
    pub fn poll(&mut self) -> io::Result<()> {
        if let Some(channel) = &self.lsp_recv {
            while let Ok(msg) = channel.try_recv() {
                use LspToEditorMessage::*;
                match msg {
                    NewLsp { lang, init_result } => self
                        .language_servers
                        .entry(lang)
                        .or_default()
                        .push(LanguageServer::new(init_result)),
                    SemanticTokens { tokens } => {
                        self.doc.semtoks = RangeSequence::from_abs_ordered(
                            self.language_servers
                                .get(&self.doc.language().unwrap())
                                .unwrap()[0]
                                .translate_semtoks(tokens, self.doc.text()),
                        );
                        self.draw()?;
                    }
                    Diagnostics { uri, diagnostics } => {
                        if self.filepath.as_ref().is_none_or(|p| {
                            lsp_types::Url::from_file_path(p.canonicalize().unwrap()).unwrap()
                                != uri
                        }) {
                            continue;
                        }
                        self.doc.diagnostics = RangeSequence::from_abs(
                            diagnostics
                                .into_iter()
                                .map(
                                    |lsp_types::Diagnostic {
                                         range: lsp_types::Range { start, end },
                                         severity,
                                         message,
                                         ..
                                     }| {
                                        (
                                            self.doc.text().byte_of_utf16_pos_saturating(
                                                Utf16Pos::from_lsp_pos(start),
                                            )
                                                ..self.doc.text().byte_of_utf16_pos_saturating(
                                                    Utf16Pos::from_lsp_pos(end),
                                                ),
                                            Diagnostic {
                                                severity: Severity::from_lsp(severity),
                                                message,
                                            },
                                        )
                                    },
                                )
                                .collect(),
                        );
                        self.draw()?;
                    }
                    Hover { view } => {
                        self.gadget = Some(Box::new(MarkdownGadget::new(view)));
                        self.draw()?;
                    }
                    Completion { items } => {
                        self.gadget = Some(Box::new(Completer::new(items)));
                        self.draw()?;
                    }
                }
            }
        }
        if let Some(chan) = &self.lsp_send
            && !self.doc.lsp_changes.is_empty()
            && let Some(lang) = self.doc.language()
            && let Some(path) = &self.filepath
        {
            _ = chan.send(EditorToLspMessage::ChangeDoc {
                lang,
                path: path.clone(),
                changes: self.doc.lsp_changes.drain(..).map(Into::into).collect(),
                version: {
                    self.doc.lsp_version += 1;
                    self.doc.lsp_version
                },
            });
        }
        Ok(())
    }
}
