use std::{collections::HashMap, sync::LazyLock};

use mutx::Mutex;
use tree_sitter::Query;

use crate::{ts::QuerySource, util::leak};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Language {
    Cpp,
    Css,
    Javascript,
    Query,
    Rust,
    Sulu,
    Toml,
    Yaml,
}

pub struct LangLspInfo {
    pub id: &'static str,
    pub command: &'static str,
}

impl Language {
    pub fn from_file_ext(ext: &str) -> Option<Self> {
        Some(match ext {
            "cc" | "cpp" | "h" | "hpp" => Self::Cpp,
            "css" => Self::Css,
            "js" => Self::Javascript,
            "tsq" => Self::Query,
            "rs" => Self::Rust,
            "sulu" => Self::Sulu,
            "toml" => Self::Toml,
            "yaml" => Self::Yaml,
            _ => None::<!>?,
        })
    }

    pub fn lsp_info(self) -> Option<LangLspInfo> {
        match self {
            Language::Cpp => Some(LangLspInfo {
                id: "cpp",
                command: "clangd",
            }),
            Language::Rust => Some(LangLspInfo {
                id: "rust",
                command: "rust-analyzer",
            }),
            _ => None,
        }
    }

    pub fn ts_lang(self) -> tree_sitter::Language {
        match self {
            Language::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            Language::Css => tree_sitter_css_orchard::LANGUAGE.into(),
            Language::Javascript => tree_sitter_javascript::LANGUAGE.into(),
            Language::Query => tree_sitter_tsquery::LANGUAGE.into(),
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
            Language::Sulu => tree_sitter_sulu::LANGUAGE.into(),
            Language::Toml => tree_sitter_toml::LANGUAGE.into(),
            Language::Yaml => tree_sitter_yaml::LANGUAGE.into(),
        }
    }

    pub fn highlight_query_source(self) -> QuerySource {
        QuerySource {
            source: match self {
                Language::Cpp => include_str!("../query/cpp/highlights.tsq"),
                Language::Css => include_str!("../query/css/highlights.tsq"),
                Language::Javascript => include_str!("../query/js/highlights.tsq"),
                Language::Query => include_str!("../query/query/highlights.tsq"),
                Language::Rust => include_str!("../query/rust/highlights.tsq"),
                Language::Sulu => include_str!("../query/sulu/highlights.tsq"),
                Language::Toml => include_str!("../query/toml/highlights.tsq"),
                Language::Yaml => include_str!("../query/yaml/highlights.tsq"),
            },
            lang: self,
        }
    }

    pub fn highlight_query(self) -> &'static Query {
        static CACHE: LazyLock<Mutex<HashMap<Language, &'static Query>>> =
            LazyLock::new(Default::default);
        CACHE
            .lock()
            .entry(self)
            .or_insert_with(|| leak(self.highlight_query_source().build().unwrap()))
    }
}