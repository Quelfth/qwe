use std::range::Range;

use tree_sitter::QueryCursor;

use crate::{
    document::{Document, diagnostics::Severity, tree::MetaQueryCapture},
    ix::{Byte, Ix},
    lang::{Highlights, Zebra},
    ts::QueryCx,
    util::{CharClass, MapBounds, word_splits},
};

pub struct Highlight {
    pub range: Range<Ix<Byte>>,
    pub scope: Scope,
    pub injection_layer: Option<u32>,
    pub priority: i32,
}

pub struct Scope(pub Vec<String>);

pub struct ScopeWithProperties {
    scope: Scope,
    ilayer: u32,
    priority: i32,
}

impl ScopeWithProperties {
    fn parse(name: &str) -> Self {
        let mut ilayer = 0;
        let mut priority = 0;
        let name = if let Some((name, rest)) = name.split_once("_") {
            if rest.starts_with(".") {
                for section in rest.split(".") {
                    if section.is_empty() || section.starts_with("_") {continue}
                    if let Some((key, value)) = section.split_once("_") {
                        match key {
                            "ilayer" => ilayer = value.parse().unwrap_or_default(),
                            "priority" => priority = value.parse().unwrap_or_default(),
                            _ => ()
                        }
                    }
                }
            }
            name
        } else {
            name
        };
        Self {
            scope: Scope(name.split(".").map(|s| s.to_owned()).collect::<Vec<_>>()),
            ilayer, priority,
        }
    }
}

impl Scope {

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
    pub fn highlight(&self, cx: &QueryCx<'_>) -> Vec<Highlight> {
        let mut highlight_scopes = Vec::new();

        macro_rules! qc {
            () => {
                &mut QueryCursor::new()
            };
        }

        if let Some(lang) = self.language() && let Some(tree) = self.tree() {
            for MetaQueryCapture { node, name, layer } in tree.query::<Highlights>(cx, qc!(), self.text(), lang) {
                let range = Range::from(node.byte_range()).map_bounds(Ix::new);
                let ScopeWithProperties { scope, ilayer, priority } = ScopeWithProperties::parse(name);
                highlight_scopes.push(Highlight {
                    scope,
                    range,
                    injection_layer: Some(layer + ilayer),
                    priority,
                });
            }
            for MetaQueryCapture { node, name, layer } in tree.query::<Zebra>(cx, qc!(), self.text(), lang) {
                if name != "zebra" {
                    continue;
                }
                let range = Range::from(node.byte_range()).map_bounds(Ix::<Byte>::new);
                let mut i = range.start;
                let mut j = i + Ix::new(1);
                let char_at =
                    |i| self.text()
                        .byte_slice(i..)
                        .unwrap()
                        .chars()
                        .next()
                        .map(CharClass::of)
                        .unwrap_or(CharClass::Symbol('\0'));
                let mut last_char = char_at(i);
                let mut even = false;
                let zebra_word = |range: Range<Ix<Byte>>, even: &mut bool, hl_scopes: &mut Vec<Highlight>| -> () {
                    let start = range.start;
                    for range in word_splits(self.text().byte_slice(range).unwrap()) {
                        if *even {
                            hl_scopes.push(Highlight {
                                scope: Scope::zebra(),
                                range: range.map_bounds(|b| b + start),
                                injection_layer: Some(layer),
                                priority: 0,
                            });
                        }
                        *even ^= true;
                    }
                };
                while i < range.end {
                    if j >= range.end {
                        zebra_word(i..j, &mut even, &mut highlight_scopes);
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
                                zebra_word(i..j-Ix::new(1), &mut even, &mut highlight_scopes);
                                i = j - Ix::new(1);
                            }
                            (Symbol(_), _) => {
                                if last_char != char {
                                    highlight_scopes.push(Highlight {
                                        scope: Scope::zebra_boundary(),
                                        range: i..j,
                                        injection_layer: Some(layer),
                                        priority: 0,
                                    });
                                    i = j;
                                }
                            }
                            _ => {
                                if last_char != char {
                                    zebra_word(i..j, &mut even, &mut highlight_scopes);
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
                range,
                scope: Scope::diagnostic(diagnostic.severity),
                injection_layer: None,
                priority: 0,
            })
        }

        highlight_scopes
    }
}
