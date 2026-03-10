use std::ops::Range;

use tree_sitter::{QueryCapture, QueryCursor};

use crate::{
    document::{Document, diagnostics::Severity},
    ix::{Byte, Ix},
    lang::{Highlights, Zebra},
    util::{CharClass, MapBounds},
};

pub mod predicate;

pub struct Highlight {
    pub range: Range<Ix<Byte>>,
    pub scope: Scope,
}

pub struct Scope(pub Vec<String>);

impl Scope {
    fn from_capture_name(name: &str) -> Self {
        let name = if let Some((name, _)) = name.split_once("_") {
            name
        } else {
            name
        };
        Self(name.split(".").map(|s| s.to_owned()).collect::<Vec<_>>())
    }

    fn diagnostic(severity: Severity) -> Self {
        Self(vec![
            "diagnostic".to_owned(),
            match severity {
                Severity::Err => "error",
                Severity::Warn => "warning",
                Severity::Info => "info",
                Severity::Hint => "hint",
            }
            .to_owned(),
        ])
    }

    fn zebra() -> Self {
        Self(vec!["zebra".to_owned()])
    }

    fn zebra_boundary() -> Self {
        Self(vec!["zebra-boundary".to_owned()])
    }
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
            let hl_query = lang.query::<Highlights>();
            for QueryCapture { node, index } in self.query_captures(qc!(), &cx, hl_query) {
                let name = hl_query.capture_names()[*index as usize];
                let range = node.byte_range().map_bounds(Ix::new);
                highlight_scopes.push(Highlight {
                    scope: Scope::from_capture_name(name),
                    range,
                });
            }
            let zebra = lang.query::<Zebra>();
            for QueryCapture { node, index } in self.query_captures(qc!(), &cx, zebra) {
                let name = zebra.capture_names()[*index as usize];
                if name != "zebra" {
                    continue;
                }
                let range = node.byte_range().map_bounds(Ix::<Byte>::new);
                let mut i = range.start;
                let mut j = i + Ix::new(1);
                let char_at =
                    |i| CharClass::of(self.text().byte_slice(i..).unwrap().chars().next().unwrap());
                let mut last_char = char_at(i);
                let mut even = false;
                while i < range.end {
                    if j >= range.end {
                        if even {
                            highlight_scopes.push(Highlight {
                                scope: Scope::zebra(),
                                range: i..j,
                            });
                        }
                        break;
                    }

                    let char = char_at(j);
                    'continu: {
                        use CharClass::*;
                        match (last_char, char) {
                            (Cap, Lower) => {
                                if j < range.start + Ix::new(2) || char_at(j - Ix::new(2)) != Cap {
                                    break 'continu;
                                }
                                if even {
                                    highlight_scopes.push(Highlight {
                                        scope: Scope::zebra(),
                                        range: i..j - Ix::new(1),
                                    });
                                }
                                even ^= true;
                                i = j - Ix::new(1);
                            }
                            (Symbol(_), _) => {
                                if last_char != char {
                                    highlight_scopes.push(Highlight {
                                        scope: Scope::zebra_boundary(),
                                        range: i..j,
                                    });
                                    i = j;
                                }
                            }
                            _ => {
                                if last_char != char {
                                    if even {
                                        highlight_scopes.push(Highlight {
                                            scope: Scope::zebra(),
                                            range: i..j,
                                        });
                                    }
                                    even ^= true;
                                    i = j;
                                }
                            }
                        }
                    }
                    j += Ix::new(1);
                    last_char = char;
                }
            }
        }

        for (range, diagnostic) in self.diagnostics.ranges() {
            highlight_scopes.push(Highlight {
                range: range.clone(),
                scope: Scope::diagnostic(diagnostic.severity),
            })
        }

        highlight_scopes
    }
}
