use std::{collections::HashMap, sync::LazyLock};

use expanda::{declare_item, expand};
use include_optional::include_str_optional;
use mutx::Mutex;
use serde_json::{Value, json};
use tree_sitter::Query;

use crate::{document::diagnostics::Severity, lsp::SpecialBehavior, ts::QuerySource, util::leak};

#[declare_item(LANGUAGE)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Language {
    Cpp,
    CSharp,
    Css,
    Javascript,
    Kotlin,
    Lua,
    Mona,
    Nu,
    Query,
    Rust,
    Sulu,
    Toml,
    Wesl,
    Yaml,

    RustFormatArgs,
}

pub struct LangLspInfo {
    pub id: &'static str,
    pub command: &'static str,
    pub args: &'static [&'static str],
    pub special_init: SpecialBehavior,
    pub options: Option<Value>,
    pub severity_map: fn(lsp_types::DiagnosticSeverity) -> Severity,
}

fn default_severity_map(severtity: lsp_types::DiagnosticSeverity) -> Severity {
    use lsp_types::DiagnosticSeverity;
    match severtity {
        DiagnosticSeverity::ERROR => Severity::Err,
        DiagnosticSeverity::WARNING => Severity::Warn,
        DiagnosticSeverity::INFORMATION => Severity::Info,
        DiagnosticSeverity::HINT => Severity::Hint,
        _ => Severity::Warn,
    }
}

impl Language {
    pub fn from_file_ext(ext: &str) -> Option<Self> {
        Self::from_identifier(ext)
    }

    pub fn from_injection_name(name: &str) -> Option<Self> {
        Self::from_identifier(name)
    }

    pub fn from_identifier(name: &str) -> Option<Self> {
        Some(match name {
            "c" | "cc" | "cpp" | "h" | "hpp" => Self::Cpp,
            "cs" | "csharp" => Self::CSharp,
            "css" => Self::Css,
            "js" | "javascript" => Self::Javascript,
            "kt" | "kotlin" => Self::Kotlin,
            "lua" => Self::Lua,
            "mn" | "mona" => Self::Mona,
            "nu" => Self::Nu,
            "tsq" => Self::Query,
            "rs" | "rust" => Self::Rust,
            "sulu" => Self::Sulu,
            "toml" => Self::Toml,
            "wgsl" | "wesl" => Self::Wesl,
            "yaml" => Self::Yaml,

            "rust-format-args" => Self::RustFormatArgs,
            _ => None::<!>?,
        })
    }

    pub fn lsp_info(self) -> Option<LangLspInfo> {
        match self {
            Language::Cpp => Some(LangLspInfo {
                id: "cpp",
                command: "clangd",
                args: &[],
                special_init: SpecialBehavior::NoOp,
                options: None,
                severity_map: default_severity_map,
            }),
            Language::Rust => Some(LangLspInfo {
                id: "rust",
                command: "rust-analyzer",
                args: &[],
                special_init: SpecialBehavior::NoOp,
                options: Some(json!{{
                    "check": {
                        "command": "clippy",
                    },
                }}),
                severity_map: |severity| {
                    match severity {
                        lsp_types::DiagnosticSeverity::ERROR => Severity::Err,
                        lsp_types::DiagnosticSeverity::WARNING => Severity::Warn,
                        lsp_types::DiagnosticSeverity::HINT => Severity::Context,
                        _ => Severity::Warn,
                    }
                },
            }),
            Language::CSharp => Some(LangLspInfo {
                id: "cs",
                command: "roslyn-language-server",
                args: &["--stdio"],
                special_init: SpecialBehavior::Roslyn,
                options: None,
                severity_map: default_severity_map,
            }),
            Language::Kotlin => Some(LangLspInfo {
                id: "kotlin",
                command: "intellij-server",
                args: &["--stdio"],
                special_init: SpecialBehavior::NoOp,
                options: None,
                severity_map: default_severity_map,
            }),
            _ => None,
        }
    }

    pub fn autosave(self) -> bool {
        use Language::*;
        matches!(self, Rust | Sulu | Query | Javascript | Wesl)
    }

    pub fn query<Q>(self) -> &'static Query
    where
        Self: LanguageQuery<Q>,
    {
        <Self as LanguageQuery<Q>>::query(self)
    }
}

impl Language {
    pub fn ts_lang(self) -> tree_sitter::Language {
        expand! {
            match self {
                <--for $pair in
                    (Cpp tree_sitter_cpp)
                    (CSharp tree_sitter_c_sharp)
                    (Css tree_sitter_css_orchard)
                    (Javascript tree_sitter_javascript)
                    (Kotlin tree_sitter_kotlin)
                    (Lua tree_sitter_lua)
                    (Mona tree_sitter_mona)
                    (Nu tree_sitter_nu)
                    (Query tree_sitter_tsquery)
                    (Rust tree_sitter_rust)
                    (Sulu tree_sitter_sulu)
                    (Toml tree_sitter_toml)
                    (Wesl tree_sitter_wesl)
                    (Yaml tree_sitter_yaml)
                    (RustFormatArgs tree_sitter_rust_format_args)
                {
                    <--let ($lang. $ts.) = $pair
                    Language::$lang => $ts::LANGUAGE.into(),
                }
            }
        }
    }

    expand! {
        <--use LANGUAGE

        <--let ($*^({$*.}). {$*($langs. ,)}) = $LANGUAGE

        pub const ALL: [Self; ${langs.len}] = [
            <--for $lang in $langs {
                Self::$lang,
            }
        ];
    }
}

pub trait LanguageQuery<Q> {
    fn query(self) -> &'static Query;
}

expand! {
    <--use LANGUAGE
    <--let env CARGO_MANIFEST_DIR

    <--let ($*^({$*.}). {$*($langs. ,)}) = $LANGUAGE

    <--for $q in
        Highlights
        Injections
        Locals
        Rulers
        Zebra
    {
        pub enum $q {}
        impl LanguageQuery<$q> for Language {
            fn query(self) -> &'static Query {
                static CACHE: LazyLock<Mutex<HashMap<Language, &'static Query>>> = LazyLock::new(Default::default);
                CACHE.lock().entry(self).or_insert_with(|| {
                    leak(
                        QuerySource {
                            source: match self {
                                <--for $lang in $langs {
                                    Language::$lang => const {
                                        match include_str_optional!(${
                                            CARGO_MANIFEST_DIR
                                            "/query/"
                                            lang.snake_case.stringify.to_dashes
                                            "/"
                                            q.snake_case.stringify.to_dashes
                                            ".tsq"
                                        }) {
                                            Some(x) => x,
                                            None => "",
                                        }
                                    },
                                }
                            },
                            lang: self,
                        }
                        .build()
                        .unwrap(),
                    )
                })
            }
        }
    }
}



