use std::io;

use crate::{editor::Editor, language_server::LanguageServer, lsp::channel::LspToEditorMessage};

impl Editor {
    pub fn poll_channels(&mut self) -> io::Result<()> {
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
        Ok(())
    }
}
