use std::{collections::HashMap, iter, ops::Deref, range::Range, sync::Arc};

use tree_sitter::{Query, QueryCapture, QueryCursor, QueryError, QueryMatch, QueryPredicate, StreamingIterator as _, Tree};

use crate::{
    document::semtoks::SemanticToken,
    ix::{Byte, Ix, Line},
    lang::Language,
    range_tree::RangeTree,
    rope::Rope,
    ts::{directive::{DirectiveError}, predicate::{Predicate, PredicateError}},
    util::{MapBounds as _, RangeOverlap as _}
};

mod predicate;
mod directive;

pub use directive::{Directive, DirectivesExt};

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
    pub locals: HashMap<tree_sitter::Node<'s>, Arc<[String]>>,
    pub semtoks: RangeTree<Ix<Byte>, &'s SemanticToken>,
    pub relevant_lines: Range<Ix<Line>>,
}

impl QueryCx<'static> {
    pub fn empty() -> Self {
        Self {
            locals: Default::default(),
            semtoks: RangeTree::default(),
            relevant_lines: Default::default(),
        }
    }
}

pub struct QueryCaptureWithDirectives<'t, 'c> {
    pub capture: &'c QueryCapture<'t>,
    pub directives: Vec<Directive>,
}

pub fn query_captures<'t, 'c>(
    tree: &'t Tree,
    text: &Rope,
    cursor: &'c mut QueryCursor,
    context: &QueryCx<'t>,
    query: &'static Query,
    cull_irrelevant: bool,
) -> impl Iterator<Item = QueryCaptureWithDirectives<'t, 'c>>
where
    't: 'c,
{
    gen move {
        let semtoks = &context.semtoks;
        let locals = &context.locals;
        let root = tree.root_node();

        let mut matches = cursor.matches(
            query,
            root,
            text,
        );

        'matches:
        while let Some(QueryMatch {
                pattern_index,
                captures,
                ..
            }) = matches.next()
        {
            if cull_irrelevant && !captures.iter().any(|QueryCapture { node, .. }| {
                let start = Ix::new(node.start_position().row);
                let end = Ix::new(node.end_position().row);
                (start..end).overlaps(context.relevant_lines)
            }) {
                continue
            }

            let mut predicates = Vec::new();
            let mut directives = Vec::new();

            for predicate in query
                .general_predicates(*pattern_index)
                .iter()
                .filter_map(|p| parse_predicate(p).ok())
            {
                match predicate {
                    PredicateOrDirective::Predicate(predicate) => predicates.push(predicate),
                    PredicateOrDirective::Directive(directive) => directives.push(directive),
                }
            }

            let capture_nodes = captures
                .iter()
                .map(|QueryCapture { node, index }| (*index, node))
                .collect::<HashMap<_, _>>();

            for pred in predicates {
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
                    Predicate::Local { capture, predicate } => {
                        let node = capture_nodes[&capture];
                        if !predicate.check(&locals.get(node).iter().flat_map(|x| &***x).map(Deref::deref).collect()) {
                            continue 'matches
                        }
                    },
                }
            }

            for capture in *captures {
                let directives = directives.iter().filter(|d| d.0 == capture.index).map(|d| d.1).collect();
                yield QueryCaptureWithDirectives { capture, directives };
            }
        }
    }
}

pub enum PredicateOrDirective {
    Predicate(Predicate),
    Directive((u32, Directive)),
}

pub enum PredicateOrDirectiveError {
    Predicate(PredicateError),
    Directive(DirectiveError),
}

impl From<PredicateError> for PredicateOrDirectiveError {
    fn from(value: PredicateError) -> Self {
        Self::Predicate(value)
    }
}

impl From<DirectiveError> for PredicateOrDirectiveError {
    fn from(value: DirectiveError) -> Self {
        Self::Directive(value)
    }
}

fn parse_predicate(predicate: &QueryPredicate) -> Result<PredicateOrDirective, PredicateOrDirectiveError> {
    let QueryPredicate { operator, args } = predicate;
    let (name, operator) = operator.split_at(operator.floor_char_boundary(operator.len()-1));
    Ok(match operator {
        "?" => PredicateOrDirective::Predicate(Predicate::parse(name, args)?),
        "!" => PredicateOrDirective::Directive(Directive::parse(name, args)?),
        _ => todo!()
    })
}
