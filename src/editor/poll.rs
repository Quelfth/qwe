use std::{io, mem};

use lsp_types::TextDocumentContentChangeEvent;

use crate::{
    editor::Editor,
    language_server::LanguageServer,
    lsp::channel::{EditorToLspMessage, LspToEditorMessage},
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
                        self.doc.semtoks = self
                            .language_servers
                            .get(&self.doc.language().unwrap())
                            .unwrap()[0]
                            .translate_semtoks(tokens, self.doc.text())
                            .collect();
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
                version: self.doc.lsp_version,
            });
        }
        Ok(())
    }
}
