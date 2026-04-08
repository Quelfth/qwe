use std::{collections::HashMap, sync::LazyLock};

use concat_into::concat_into;
use include_optional::include_str_optional;
use mutx::Mutex;
use tree_sitter::Query;

use crate::{ts::QuerySource, util::leak};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Language {
    Cpp,
    Css,
    Javascript,
    Mona,
    Nu,
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
            "c" | "cc" | "cpp" | "h" | "hpp" => Self::Cpp,
            "css" => Self::Css,
            "js" => Self::Javascript,
            "mn" => Self::Mona,
            "nu" => Self::Nu,
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

    pub fn query<Q>(self) -> &'static Query
    where
        Self: LanguageQuery<Q>,
    {
        <Self as LanguageQuery<Q>>::query(self)
    }
}

queries! {
     {
        Cpp => "cpp"
        Css => "css"
        Javascript => "js"
        Mona => "mona"
        Nu => "nu"
        Query => "query"
        Rust => "rust"
        Sulu => "sulu"
        Toml => "toml"
        Yaml => "yaml"
    }

    Highlights "highlights"
    Zebra "zebra"
}

ts_lang! {
    Cpp => tree_sitter_cpp
    Css => tree_sitter_css_orchard
    Javascript => tree_sitter_javascript
    Mona => tree_sitter_mona
    Nu => tree_sitter_nu
    Query => tree_sitter_tsquery
    Rust => tree_sitter_rust
    Sulu => tree_sitter_sulu
    Toml => tree_sitter_toml
    Yaml => tree_sitter_yaml
}

pub trait LanguageQuery<Q> {
    fn query(self) -> &'static Query;
}

macro_rules! queries {
    ($m:tt $($q:ident $file:literal)* ) => { $(
        query! {
            $q $file $m
        }
    )* }
}
use queries;

macro_rules! query {
    ($q:ident $file:literal { $($lang:ident => $dir:literal )* }) => {
        pub enum $q {}
        impl LanguageQuery<$q> for Language {
            fn query(self) -> &'static Query {
                static CACHE: LazyLock<Mutex<HashMap<Language, &'static Query>>> =
                    LazyLock::new(Default::default);
                CACHE.lock().entry(self).or_insert_with(|| {
                    leak(
                        QuerySource {
                            source: match self { $(
                                Language::$lang => const {
                                    match concat_into!(CARGO_MANIFEST_DIR "/query/" $dir "/" $file ".tsq" => include_str_optional) {
                                        Some(x) => x,
                                        None => "",
                                    }
                                },
                            )* },
                            lang: self,
                        }
                        .build()
                        .unwrap(),
                    )
                })
            }
        }
    };
}
use query;

macro_rules! ts_lang {
    ($($lang:ident => $ts_lang:ident)*) => {
        impl Language {
            pub fn ts_lang(self) -> tree_sitter::Language {
                match self {
                    $(Language::$lang => $ts_lang::LANGUAGE.into(),)*
                }
            }
        }
    };
}
use ts_lang;
