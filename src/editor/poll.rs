use std::{convert::identity, io};

use crossterm::event::MouseEvent;
use lsp_types::Url;

use crate::{
    AppState, document::diagnostics::{Diagnostic, Severity}, editor::{
        Editor, code_actions::{CodeAction, CodeActionsGadget}, completer::Completer, keymap::InputEvent, markdown_view::MarkdownGadget, picker::Picker
    }, language_server::LanguageServer, lsp::channel::{EditorToLspMessage, LspToEditorMessage}, pos::Utf16Pos, presenter::Present, range_sequence::RangeSequence
};

impl AppState for Editor {
    fn poll(&mut self) -> io::Result<()> {
        let mut action = None::<Box<dyn FnOnce(&mut Editor) -> io::Result<()>>>;
        if let Some(cx) = &self.lsp {
            while let Ok(msg) = cx.rx.try_recv() {
                use LspToEditorMessage::*;
                match msg {
                    NewLsp { lang, init_result } => {
                        cx.servers.lock()
                            .entry(lang)
                            .or_default()
                            .push(LanguageServer::new(init_result))
                    },
                    SemanticTokens { uri, tokens } => {
                        if uri.scheme() == "file"
                            && uri.to_file_path().is_ok_and(|p| {
                                self.filepath
                                    .as_ref()
                                    .and_then(|f| {
                                        Some(f.canonicalize().ok()? == p.canonicalize().ok()?)
                                    })
                                    .is_some_and(identity)
                            })
                        {
                            self.doc.semtoks = RangeSequence::from_abs_ordered(
                                cx.servers.lock()
                                    .get(&self.doc.language().unwrap())
                                    .unwrap()[0]
                                    .translate_semtoks(tokens, self.doc.text()),
                            );
                            self.presenter.defer_draw();
                        }
                    }
                    Diagnostics { uri, diagnostics } => {
                        let Some(path) = self.filepath.clone() else {continue};
                        let Ok(path) = path.canonicalize() else {continue};
                        let doc = if let Ok(x) = Url::from_file_path(&path) && x == uri {
                            self.presenter.defer_draw();
                            &mut self.doc
                        } else if let Some(doc) = self.bg_docs.by_path_mut(&path) {
                            doc
                        } else { continue };

                        doc.diagnostics = RangeSequence::from_abs(
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
                                            doc.text().byte_of_utf16_pos_saturating(
                                                Utf16Pos::from_lsp_pos(start),
                                            )
                                                ..doc.text().byte_of_utf16_pos_saturating(
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
                            {
                                let pos = Utf16Pos::from_lsp_pos(range.start);
                                action = Some(Box::new(move |e: &mut Self| -> Result<(), _> {
                                    _= e.open_file_doc_at(path.into(), pos);
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
        if let Some(cx) = &self.lsp
            && !self.doc.lsp_changes.is_empty()
            && let Some(lang) = self.doc.language()
            && let Some(path) = &self.filepath
        {
            _ = cx.tx.send(EditorToLspMessage::ChangeDoc {
                lang,
                path: path.clone(),
                changes: self.doc.lsp_changes.drain(..).map(Into::into).collect(),
                version: {
                    self.doc.lsp_version += 1;
                    self.doc.lsp_version
                },
            });
        }
        self.poll_draw()?;
        Ok(())
    }

    fn on_key_event(&mut self, event: InputEvent) -> io::Result<()>{
        self.on_key_event(event)
    }

    fn on_mouse_event(&mut self, event: MouseEvent) -> io::Result<()>{
        self.on_mouse_event(event)
    }
}
