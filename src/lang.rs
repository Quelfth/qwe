use crate::ts::QuerySource;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Language {
    Rust,
}

impl Language {
    pub fn from_file_ext(ext: &str) -> Option<Self> {
        Some(match ext {
            "rs" => Self::Rust,
            _ => None::<!>?,
        })
    }

    pub fn ts_lang(self) -> tree_sitter::Language {
        match self {
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
        }
    }

    pub fn highlight_query_source(self) -> QuerySource {
        QuerySource {
            source: match self {
                Language::Rust => include_str!("../query/rust/highlights.scm"),
            },
            lang: self,
        }
    }
}
