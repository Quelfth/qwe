use std::{collections::HashMap, ops::Range, sync::Arc};

use convert_case::{Case, Casing};
use lsp_types::{
    InitializeResult, SemanticTokensLegend, SemanticTokensOptions,
    SemanticTokensRegistrationOptions, SemanticTokensServerCapabilities,
};
use mutx::Mutex;

use crate::{
    document::semtoks::SemanticToken, ix::{Byte, Ix, Line, Utf16}, lang::Language, lsp::channel::{EditorToLspMessage, LspToEditorMessage}, rope::Rope
};

#[derive(PartialEq, Eq, Default)]
pub enum TextEncoding {
    #[default]
    Utf16,
    Utf8,
}

pub struct LanguageServer {
    _name: Option<String>,
    encoding: TextEncoding,
    semtok_legend: Option<Legend>,
}

struct Legend {
    types: Vec<Arc<str>>,
    mods: Vec<Arc<str>>,
}

impl From<SemanticTokensLegend> for Legend {
    fn from(value: SemanticTokensLegend) -> Self {
        Self {
            types: value
                .token_types
                .into_iter()
                .map(|t| t.as_str().to_case(Case::UpperCamel).into())
                .collect(),
            mods: value
                .token_modifiers
                .into_iter()
                .map(|t| t.as_str().to_case(Case::Kebab).into())
                .collect(),
        }
    }
}

impl LanguageServer {
    pub fn new(init: InitializeResult) -> Self {
        let InitializeResult {
            capabilities,
            server_info,
        } = init;

        let legend = if let Some(tokens) = capabilities.semantic_tokens_provider {
            let (SemanticTokensServerCapabilities::SemanticTokensOptions(options)
            | SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                SemanticTokensRegistrationOptions {
                    semantic_tokens_options: options,
                    ..
                },
            )) = tokens;
            let SemanticTokensOptions { legend, .. } = options;
            Some(legend)
        } else {
            None
        };

        let encoding = if let Some(enc) = capabilities.position_encoding {
            match enc.as_str() {
                "utf-16" => TextEncoding::Utf16,
                "utf-8" => TextEncoding::Utf8,
                _ => TextEncoding::default(),
            }
        } else {
            TextEncoding::default()
        };

        let name = server_info.map(|i| i.name);
        Self {
            _name: name,
            encoding,
            semtok_legend: legend.map(Into::into),
        }
    }
}

impl LanguageServer {
    pub fn translate_semtoks(
        &self,
        tokens: Vec<lsp_types::SemanticToken>,
        text: &Rope,
    ) -> impl Iterator<Item = (Range<Ix<Byte>>, SemanticToken)> {
        if self.semtok_legend.is_none() {
            panic!()
        }
        if self.encoding != TextEncoding::Utf16 {
            todo!()
        }
        let mut line: Ix<Line> = Ix::new(0);
        let mut pos: Ix<Byte> = Ix::new(0);
        tokens.into_iter().map(
            move |lsp_types::SemanticToken {
                      delta_line,
                      delta_start,
                      length,
                      token_type,
                      token_modifiers_bitset,
                  }| {
                let delta_line: Ix<Line> = Ix::new(delta_line as _);
                let delta_start: Ix<Utf16> = Ix::new(delta_start as _);
                let len: Ix<Utf16> = Ix::new(length as _);
                line += delta_line;
                if delta_line > Ix::new(0) {
                    pos = text.byte_of_line(line).unwrap_or(text.byte_len());
                }
                if let Some(line) = text.line(line) {
                    pos += line.byte_of_utf16_saturating(delta_start);
                }

                let range = pos..pos + try {text.byte_slice(pos..)?.byte_of_utf16_saturating(len)}.unwrap_or(Ix::new(0));

                let legend = self.semtok_legend.as_ref().unwrap();

                let r#type = legend.types[token_type as usize].clone();
                let mods = (0..32)
                    .filter(|i| (token_modifiers_bitset & 1 << i) != 0)
                    .map(|i| legend.mods[i as usize].clone())
                    .collect::<Vec<_>>();

                (range, SemanticToken { r#type, mods })
            },
        )
    }
}


pub struct LspContext {
    pub rx: std::sync::mpsc::Receiver<LspToEditorMessage>,
    pub tx: tokio::sync::mpsc::UnboundedSender<EditorToLspMessage>,
    pub servers: Mutex<HashMap<Language, Vec<LanguageServer>>>,
}

impl LspContext {
    pub fn new(
        rx: std::sync::mpsc::Receiver<LspToEditorMessage>,
        tx: tokio::sync::mpsc::UnboundedSender<EditorToLspMessage>
    ) -> Self {
        Self {
            rx,
            tx,
            servers: Default::default(),
        }
    }
}