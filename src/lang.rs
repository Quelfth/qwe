use crate::ts::QuerySource;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Language {
    Rust,
    Query,
    Sulu,
}

impl Language {
    pub fn from_file_ext(ext: &str) -> Option<Self> {
        Some(match ext {
            "rs" => Self::Rust,
            "tsq" => Self::Query,
            "sulu" => Self::Sulu,
            _ => None::<!>?,
        })
    }

    pub fn ts_lang(self) -> tree_sitter::Language {
        match self {
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
            Language::Query => tree_sitter_tsquery::LANGUAGE.into(),
            Language::Sulu => tree_sitter_sulu::LANGUAGE.into(),
        }
    }

    pub fn highlight_query_source(self) -> QuerySource {
        QuerySource {
            source: match self {
                Language::Rust => include_str!("../query/rust/highlights.tsq"),
                Language::Query => include_str!("../query/query/highlights.tsq"),
                Language::Sulu => include_str!("../query/sulu/highlights.tsq"),
            },
            lang: self,
        }
    }
}
