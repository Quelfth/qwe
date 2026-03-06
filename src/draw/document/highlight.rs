use std::ops::Range;

use tree_sitter::{QueryCapture, QueryCursor};

use crate::{
    document::{Document, diagnostics::Severity},
    ix::{Byte, Ix},
    lang::Highlights,
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
        let cx = self.query_capture_context();

        macro_rules! qc {
            () => {
                &mut QueryCursor::new()
            };
        }

        if let Some(lang) = self.language() {
            let query = lang.query::<Highlights>();
            for QueryCapture { node, index } in self.query_captures(qc!(), &cx, query) {
                let name = query.capture_names()[*index as usize];
                let range = node.byte_range().map_bounds(Ix::new);
                highlight_scopes.push(Highlight {
                    scope: name.split(".").map(|s| s.to_owned()).collect::<Vec<_>>(),
                    range,
                });
            }
        }

        for (range, diagnostic) in self.diagnostics.ranges() {
            let severity = match diagnostic.severity {
                Severity::Err => "error",
                Severity::Warn => "warning",
                Severity::Info => "info",
                Severity::Hint => "hint",
            }
            .to_owned();

            highlight_scopes.push(Highlight {
                range: range.clone(),
                scope: vec!["diagnostic".to_owned(), severity],
            })
        }

        highlight_scopes
    }
}
