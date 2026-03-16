use std::{io, time::Instant};

use crate::{
    PathedFile,
    document::diagnostics::{Diagnostic, Severity},
    editor::{
        Editor,
        code_actions::{CodeAction, CodeActionsGadget},
        completer::Completer,
        markdown_view::MarkdownGadget,
        picker::Picker,
    },
    language_server::LanguageServer,
    lsp::channel::{EditorToLspMessage, LspToEditorMessage},
    pos::Utf16Pos,
    range_sequence::RangeSequence,
};

impl Editor {
    pub fn poll(&mut self) -> io::Result<()> {
        let mut action = None::<Box<dyn FnOnce(&mut Editor) -> io::Result<()>>>;
        if let Some(channel) = &self.lsp_recv {
            while let Ok(msg) = channel.try_recv() {
                use LspToEditorMessage::*;
                match msg {
                    NewLsp { lang, init_result } => self
                        .language_servers
                        .entry(lang)
                        .or_default()
                        .push(LanguageServer::new(init_result)),
                    SemanticTokens { uri, tokens } => {
                        if uri.scheme() == "file"
                            && uri.to_file_path().is_ok_and(|p| {
                                self.filepath
                                    .as_ref()
                                    .and_then(|f| {
                                        Some(f.canonicalize().ok()? == p.canonicalize().ok()?)
                                    })
                                    .is_some_and(std::convert::identity)
                            })
                        {
                            self.doc.semtoks = RangeSequence::from_abs_ordered(
                                self.language_servers
                                    .get(&self.doc.language().unwrap())
                                    .unwrap()[0]
                                    .translate_semtoks(tokens, self.doc.text()),
                            );
                            self.defer_draw();
                        }
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
                        self.defer_draw();
                    }
                    Hover { view } => {
                        self.gadget = Some(Box::new(MarkdownGadget::new(view)));
                        self.draw()?;
                    }
                    Completion { items } => {
                        self.gadget = Some(Box::new(Completer::new(items)));
                        self.draw()?;
                    }
                    Goto { locations } => match &*locations {
                        [] => (),
                        [location] => {
                            let lsp_types::Location { uri, range } = location;
                            if uri.scheme() == "file"
                                && let Ok(path) = uri.to_file_path()
                                && let Ok(file) = PathedFile::open(path.into())
                            {
                                let pos = Utf16Pos::from_lsp_pos(range.start);
                                action = Some(Box::new(move |e: &mut Self| -> Result<(), _> {
                                    e.open_new_doc_at(file, pos);
                                    e.draw()
                                }));
                            }
                        }
                        locations => {
                            self.gadget = Some(Box::new(Picker::locations(locations)));
                            self.draw()?
                        }
                    },
                    CodeActions { actions } => {
                        self.gadget = Some(Box::new(CodeActionsGadget::new(
                            actions.into_iter().map(CodeAction::from_lsp).collect(),
                        )));
                        self.draw()?
                    }
                }
            }
        }
        if let Some(action) = action {
            action(self)?
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
        {
            let guard = self.draw_defer.lock();
            if let Some(defer) = *guard
                && defer <= Instant::now()
            {
                drop(guard);
                self.draw()?;
            }
        }
        Ok(())
    }
}
