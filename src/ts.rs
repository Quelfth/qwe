use thiserror::Error;
use tree_sitter::{LanguageError, Parser, Query, QueryError, Tree};

use crate::{lang::Language, rope::Rope};

#[derive(Debug, Error)]
pub enum ParseDocError {
    #[error("{0}")]
    Language(#[from] LanguageError),
    #[error("no tree was parsed")]
    NoTree,
}

pub fn parse_doc(
    text: &Rope,
    tree: Option<&Tree>,
    language: Language,
) -> Result<Tree, ParseDocError> {
    use ParseDocError as E;
    let mut parser = Parser::new();

    parser.set_language(&language.ts_lang())?;

    let tree = parser
        .parse_with_options(&mut text.ts_callback(), tree, None)
        .ok_or(E::NoTree)?;

    Ok(tree)
}

#[derive(Copy, Clone)]
pub struct QuerySource {
    pub source: &'static str,
    pub lang: Language,
}

impl QuerySource {
    pub fn build(self) -> Result<Query, QueryError> {
        let Self { source, lang } = self;
        Query::new(&lang.ts_lang(), source)
    }
}
