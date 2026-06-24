use std::{collections::HashMap, iter, range::Range};

use thiserror::Error;
use tree_sitter::{LanguageError, Parser, Query, QueryCapture, QueryCursor, QueryError, QueryMatch, StreamingIterator as _, Tree};

use crate::{document::{semtoks::SemanticToken, tree::MetaTree}, ix::{Byte, Ix, Line}, lang::Language, range_tree::RangeTree, rope::Rope, ts::predicate::Predicate, util::{MapBounds as _, RangeOverlap as _}};

mod predicate;

#[derive(Debug, Error)]
pub enum ParseDocError {
    #[error("{0}")]
    Language(#[from] LanguageError),
    #[error("no tree was parsed")]
    NoTree,
}

pub fn parse_doc(
    text: &Rope,
    tree: Option<&MetaTree>,
    language: Language,
) -> Result<MetaTree, ParseDocError> {
    use ParseDocError as E;
    let mut parser = Parser::new();

    parser.set_language(&language.ts_lang())?;

    let tree = parser
        .parse_with_options(&mut text.ts_callback(), tree.as_ref().map(|t| &t.tree), None)
        .ok_or(E::NoTree)?;

    Ok(MetaTree::simple(tree))
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

pub struct QueryCx<'s> {
    pub semtoks: RangeTree<Ix<Byte>, &'s SemanticToken>,
    pub screen_lines: Range<Ix<Line>>,
}

impl QueryCx<'static> {
    pub fn empty() -> Self {
        Self {
            semtoks: RangeTree::default(),
            screen_lines: Default::default(),
        }
    }
}

pub fn query_captures<'t, 'c>(
    tree: &'t Tree,
    text: &Rope,
    cursor: &'c mut QueryCursor,
    context: &QueryCx<'t>,
    query: &'static Query,
    cull_offscreen: bool,
) -> impl Iterator<Item = &'c QueryCapture<'t>>
where
    't: 'c,
{
    gen move {
        let semtoks = &context.semtoks;
        let root = tree.root_node();

        let mut matches = cursor.matches_with_options(
            query,
            root,
            text,
            tree_sitter::QueryCursorOptions {
                progress_callback: None,
            },
        );

        'matches:
        while let Some(QueryMatch {
                pattern_index,
                captures,
                ..
            }) = matches.next()
        {
            if cull_offscreen && !captures.iter().any(|QueryCapture { node, .. }| {
                let start = Ix::new(node.start_position().row);
                let end = Ix::new(node.end_position().row);
                (start..end).overlaps(context.screen_lines)
            }) {
                continue
            }

            let preds = query
                .general_predicates(*pattern_index)
                .iter()
                .filter_map(|p| Predicate::parse(p).ok())
                .collect::<Vec<_>>();
            let capture_nodes = captures
                .iter()
                .map(|QueryCapture { node, index }| (*index, node))
                .collect::<HashMap<_, _>>();

            for pred in preds {
                match pred {
                    Predicate::Semantic { capture, predicate } => {
                        let node = capture_nodes[&capture];
                        if !semtoks
                            .overlapping(Range::from(node.byte_range()).map_bounds(Ix::new))
                            .any(|SemanticToken { r#type, mods }| {
                                predicate.check(
                                    &iter::once(r#type.clone())
                                        .chain(mods.iter().cloned())
                                        .collect(),
                                )
                            })
                        {
                            continue 'matches;
                        }
                    }
                }
            }

            for capture in *captures {
                yield capture;
            }
        }
    }
}