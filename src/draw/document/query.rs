use std::{collections::HashMap, iter};

use tree_sitter::{Query, QueryCapture, QueryCursor, QueryMatch, StreamingIterator};

use crate::{
    document::{Document, semtoks::SemanticToken},
    ix::{Byte, Ix},
    range_tree::RangeTree,
    util::MapBounds,
};

use super::highlight::predicate;
use predicate::Predicate;

pub struct QueryCaptureContext<'s> {
    semtoks: RangeTree<Ix<Byte>, &'s SemanticToken>,
}

impl Document {
    pub fn query_capture_context(&self) -> QueryCaptureContext<'_> {
        QueryCaptureContext {
            semtoks: self.semtoks.ranges().collect::<RangeTree<_, _>>(),
        }
    }

    pub fn query_captures<'s, 'c, 'x, 'r>(
        &'s self,
        cursor: &'c mut QueryCursor,
        context: &'x QueryCaptureContext<'s>,
        query: &'static Query,
    ) -> impl Iterator<Item = &'c QueryCapture<'s>>
    where
        's: 'r + 'c,
        'c: 'r,
    {
        gen move {
            let semtoks = &context.semtoks;
            if let Some(tree) = self.tree() {
                let root = tree.root_node();

                let mut matches = cursor.matches_with_options(
                    query,
                    root,
                    self.text(),
                    tree_sitter::QueryCursorOptions {
                        progress_callback: None,
                    },
                );

                'matches: while let Some(QueryMatch {
                    pattern_index,
                    captures,
                    ..
                }) = matches.next()
                {
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
                                    .overlapping(node.byte_range().map_bounds(Ix::new))
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
    }
}
