use std::{collections::{BTreeMap, HashMap, hash_map}, range::Range, sync::Arc};

use tree_sitter::{Node, QueryCapture, QueryCursor, QueryMatch, StreamingIterator as _};

use crate::{document::Document, ix::{Byte, Ix, ix}, lang::Locals, log::{DisplayLog, LogCategory, log}, util::{MapBounds as _, RangeOverlap as _}};

impl Document {
    pub fn resolve_locals<'s>(&'s self) -> HashMap<Node<'s>, Arc<[String]>> {
        try {
            let lang = self.language()?;
            let tree = self.tree()?;

            let mut cursor = QueryCursor::new();

            let query = lang.query::<Locals>();

            let mut matches = cursor.matches(
                query,
                tree.tree.root_node(),
                self.text(),
            );

            #[derive(Default, Debug)]
            struct Declaration {
                types: Arc<[String]>,
                scopes: Arc<[Range<Ix<Byte>>]>,
            }


            let mut decls = HashMap::<(&str, String), BTreeMap<Ix<Byte>, Declaration>>::new();
            let mut locals = HashMap::<Node<'s>, Arc<[String]>>::new();

            while let Some(QueryMatch { captures, .. }) = matches.next() {
                let scopes = captures
                    .iter()
                    .filter_map(|c| match query.capture_names()[c.index as usize] {
                        "scope" => Some(Range::from(c.node.byte_range()).map_bounds(ix)),
                        "scope.after" => Some({
                            let range = c.node.byte_range();
                            let parent_range = c.node.parent()?.byte_range();
                            ix(range.end)..ix(parent_range.end)
                        }),
                        _ => None
                    })
                    .collect::<Arc<[_]>>();

                for QueryCapture { node, index } in *captures {
                    let kind = query.capture_names()[*index as usize];
                    let range: Range<Ix<Byte>> = Range::from(node.byte_range()).map_bounds(ix);
                    if let Some(declare) = kind.strip_prefix("declare.") {

                        let name = self.text().byte_slice(range).unwrap().to_string();
                        let (namespace, declare) = declare.split_once(".").unwrap_or(("", declare));

                        let entry = decls.entry((namespace, name)).or_default();
                        let types = declare.split("_").filter(|r#type| !r#type.is_empty()).map(|s| s.to_owned()).collect::<Arc<[_]>>();

                        entry.insert(range.start, Declaration {
                            types: types.clone(),
                            scopes: scopes.clone(),
                        });

                        locals.insert(*node, types);
                    }
                }
            }

            log!(DisplayLog {
                category: LogCategory::Debug,
                message: "resolved local declarations",
                details: format!("{decls:#?}"),
            });

            let mut matches = cursor.matches(
                query,
                tree.tree.root_node(),
                self.text(),
            );


            while let Some(QueryMatch { captures, .. }) = matches.next() {
                for QueryCapture { node, index } in *captures {
                    let kind = query.capture_names()[*index as usize];
                    let (kind, namespace) = kind.split_once(".").unwrap_or((kind, ""));
                    if kind != "reference" { continue }

                    let range: Range<Ix<Byte>> = Range::from(node.byte_range()).map_bounds(ix);
                    let name = self.text().byte_slice(range).unwrap().to_string();

                    let Some(entry) = decls.get(&(namespace, name)) else {continue};

                    for Declaration { types, scopes } in entry.values().rev() {
                        if scopes.iter().any(|s| s.overlaps(range)) {
                            match locals.entry(*node) {
                                hash_map::Entry::Occupied(mut entry) => {
                                    entry.insert(entry.get().iter().chain(types.iter()).cloned().collect());
                                },
                                hash_map::Entry::Vacant(entry) => {
                                    entry.insert(types.clone());
                                },
                            }
                            break;
                        }
                    }
                }
            }

            log!(DisplayLog {
                category: LogCategory::Debug,
                message: "resolved locals",
                details: format!("{locals:#?}"),
            });

            locals
        }.unwrap_or_default()
    }
}
