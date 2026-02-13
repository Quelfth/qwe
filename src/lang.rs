use std::{collections::HashMap, sync::LazyLock};

use dashmap::DashMap;
use mutx::Mutex;
use tree_sitter::Query;

use crate::{ts::QuerySource, util::leak};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Language {
    Css,
    Query,
    Rust,
    Sulu,
    Yaml,
}

pub struct LangLspInfo {
    pub id: &'static str,
    pub command: &'static str,
}

impl Language {
    pub fn from_file_ext(ext: &str) -> Option<Self> {
        Some(match ext {
            "css" => Self::Css,
            "tsq" => Self::Query,
            "rs" => Self::Rust,
            "sulu" => Self::Sulu,
            "yaml" => Self::Yaml,
            _ => None::<!>?,
        })
    }

    pub fn lsp_info(self) -> Option<LangLspInfo> {
        match self {
            Language::Css => None,
            Language::Query => None,
            Language::Rust => Some(LangLspInfo {
                id: "rust",
                command: "rust-analyzer",
            }),
            Language::Sulu => None,
            Language::Yaml => None,
        }
    }

    pub fn ts_lang(self) -> tree_sitter::Language {
        match self {
            Language::Css => tree_sitter_css_orchard::LANGUAGE.into(),
            Language::Query => tree_sitter_tsquery::LANGUAGE.into(),
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
            Language::Sulu => tree_sitter_sulu::LANGUAGE.into(),
            Language::Yaml => tree_sitter_yaml::LANGUAGE.into(),
        }
    }

    pub fn highlight_query_source(self) -> QuerySource {
        QuerySource {
            source: match self {
                Language::Css => include_str!("../query/css/highlights.tsq"),
                Language::Query => include_str!("../query/query/highlights.tsq"),
                Language::Rust => include_str!("../query/rust/highlights.tsq"),
                Language::Sulu => include_str!("../query/sulu/highlights.tsq"),
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
