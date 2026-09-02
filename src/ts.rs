use std::{collections::HashMap, iter, ops::Deref, range::Range, sync::Arc};

use tree_sitter::{Query, QueryCapture, QueryCursor, QueryError, QueryMatch, StreamingIterator as _, Tree};

use crate::{
    document::semtoks::SemanticToken,
    ix::{Byte, Ix, Line},
    lang::Language,
    range_tree::RangeTree,
    rope::Rope,
    ts::predicate::Predicate,
    util::{MapBounds as _, RangeOverlap as _}
};

mod predicate;

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

pub fn query_captures<'t, 'c>(
    tree: &'t Tree,
    text: &Rope,
    cursor: &'c mut QueryCursor,
    context: &QueryCx<'t>,
    query: &'static Query,
    cull_irrelevant: bool,
) -> impl Iterator<Item = &'c QueryCapture<'t>>
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
                    Predicate::Local { capture, predicate } => {
                        let node = capture_nodes[&capture];
                        if !predicate.check(&locals.get(node).iter().flat_map(|x| &***x).map(Deref::deref).collect()) {
                            continue 'matches
                        }
                    },
                }
            }

            for capture in *captures {
                yield capture;
            }
        }
    }
}