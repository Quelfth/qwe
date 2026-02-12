use std::{collections::HashMap, iter, ops::Range};

use tree_sitter::{QueryCapture, QueryCursor, QueryMatch, StreamingIterator};

use crate::{
    document::{Document, semtoks::SemanticToken},
    draw::document::highlight::predicate::Predicate,
    ix::{Byte, Ix},
    util::MapBounds,
};

pub mod predicate;

pub struct Highlight {
    pub range: Range<Ix<Byte>>,
    pub scope: Vec<String>,
}

impl Document {
    pub fn highlight(&self) -> Vec<Highlight> {
        let mut highlight_scopes = Vec::new();

        if let (Some(lang), Some(tree)) = (self.language(), self.tree()) {
            let mut cursor = QueryCursor::new();
            let root = tree.root_node();

            let query = lang.highlight_query_source().build().unwrap();

            let mut matches = cursor.matches_with_options(
                &query,
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
                            if !self
                                .semtoks
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

                for QueryCapture { node, index } in *captures {
                    let name = query.capture_names()[*index as usize];
                    let range = node.byte_range().map_bounds(Ix::new);
                    highlight_scopes.push(Highlight {
                        scope: name.split(".").map(|s| s.to_owned()).collect::<Vec<_>>(),
                        range,
                    });
                }
            }
        }
        highlight_scopes
    }
}
