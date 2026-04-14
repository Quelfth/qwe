use std::{io, path::Path, sync::Arc};

use crossterm::event::MouseEvent;
use lsp_types::Url;

use crate::{
    AppState, document::{Document, diagnostics::{Diagnostic, Severity}}, editor::{
        Editor,
        code_actions::{ActionEdit, CodeAction, CodeActionsGadget},
        completer::Completer,
        keymap::InputEvent,
        markdown_view::MarkdownGadget,
        picker::Picker,
        renamer::Renamer,
    }, language_server::LanguageServer, log::log, lsp::channel::{EditorToLspMessage, LspToEditorMessage}, pos::Utf16Pos, presenter::Present, range_sequence::RangeSequence, util::{MapBounds, uri_to_canon_path}
};

impl AppState for Editor {
    fn poll(&mut self) -> io::Result<()> {
        let mut action = None::<Box<dyn FnOnce(&mut Editor) -> io::Result<()>>>;
        if let Some(cx) = &self.lsp {
            while let Ok(msg) = cx.rx.try_recv() {
                log!(msg);
                use LspToEditorMessage::*;
                match msg {
                    NewLsp { lang, init_result } => {
                        cx.servers.lock()
                            .entry(lang)
                            .or_default()
                            .push(LanguageServer::new(init_result))
                    }
                    SemanticTokens { uri, tokens } => {
                        let Some(path) = uri_to_canon_path(uri) else {continue};
                        if self.filepath
                            .as_deref()
                            .is_some_and(|p|
                                p.canonicalize()
                                    .is_ok_and(|p| p == &*path)
                            )
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
                    PrepareRename { range, text } => {
                        let name = if let Some(range) = range {
                            let range = range.map_bounds(|b|
                                self
                                    .doc()
                                    .text()
                                    .byte_of_utf16_pos_saturating(b)
                            );

                            self.doc().text().byte_slice(range).map(|s| s.to_string())
                        } else { None };
                        self.gadget = Some(Box::new(Renamer::new(
                            text
                                .or_else(|| name)
                                .unwrap_or_default()
                        )));
                        self.draw()?
                    },
                    Rename { edit } => {
                        let edits = ActionEdit::from_workspace_edit(edit);
                        action = Some(Box::new(move |e: &mut Self| -> Result<(), _> {
                            e.apply_action_edits(edits);
                            e.draw()
                        }));
                    },
                }
            }
        }
        if let Some(action) = action {
            action(self)?
        }
        if let Some(lsp) = &self.lsp {
            let send_doc_updates = |path: Arc<Path>, doc: &mut Document| {
                if !doc.lsp_changes.is_empty()
                    && let Some(lang) = doc.language() {
                    _ = lsp.tx.send(EditorToLspMessage::ChangeDoc {
                        lang,
                        path: path.clone(),
                        changes: doc.lsp_changes.drain(..).map(Into::into).collect(),
                        version: {
                            doc.lsp_version += 1;
                            doc.lsp_version
                        },
                    });
                }
            };
            if let Some(path) = &self.filepath {
                send_doc_updates(path.clone(), &mut self.doc);
            }
            for (path, doc) in self.bg_docs.pathed_mut() {
                send_doc_updates(path, doc);
            }
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
