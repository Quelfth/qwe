use std::range::Range;

use tree_sitter::QueryCursor;

use crate::{
    document::{Document, diagnostics::Severity, tree::MetaQueryCapture},
    ix::{Byte, Ix, ix},
    lang::{Highlights, Zebra},
    ts::{DirectivesExt as _, QueryCx},
    util::{CharClass, MapBounds, word_splits},
};

pub struct Highlight {
    pub range: Range<Ix<Byte>>,
    pub scope: Scope,
    pub injection_layer: Option<u32>,
    pub priority: i32,
}

#[derive(Clone)]
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
                Severity::Context => "context",
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
            for MetaQueryCapture { node, name, layer, directives } in tree.query::<Highlights>(cx, qc!(), self.text(), lang) {
                let range = Range::from(node.byte_range()).map_bounds(Ix::new);
                let ScopeWithProperties { scope, ilayer, priority } = ScopeWithProperties::parse(name);
                for range in directives.apply_to_range(range, self.text()) {
                    highlight_scopes.push(Highlight {
                        scope: scope.clone(),
                        range,
                        injection_layer: Some(layer + ilayer),
                        priority,
                    });
                }
            }
            for MetaQueryCapture { node, name, layer, directives: _ } in tree.query::<Zebra>(cx, qc!(), self.text(), lang) {
                let mut digit_limit = usize::MAX;
                let mut digit_right_align = true;
                let mut dyn_align = true;
                for rule in name.split(".").collect::<Vec<_>>() {
                    if let Some(limit) = rule.strip_suffix("digit") {
                        digit_limit = limit.parse().unwrap_or(0);
                    }
                    if rule == "digit-left-align" {
                        digit_right_align = false;
                        dyn_align = false;
                    }
                    if rule == "digit-right-align" {
                        dyn_align = false;
                    }
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
                if self.text().byte_slice(range.start..range.start + ix(2)).is_some_and(|slice| &slice == "0.") {
                    even = true;
                }
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
                let zebra_word_boundary = |range: Range<Ix<Byte>>, hl_scopes: &mut Vec<Highlight>| {
                    hl_scopes.push(Highlight {
                        scope: Scope::zebra_boundary(),
                        range,
                        injection_layer: Some(layer),
                        priority: 0,
                    });
                };
                let mut digits = 0;
                let alignment = |j: Ix<Byte>| -> usize {
                    let mut k = j + ix(1);
                    while char_at(k) == Number { k += ix(1); }
                    let count = (k - j).0;
                    let rem = count % digit_limit;
                    if rem != 0 {
                        digit_limit - rem
                    } else {
                        0
                    }
                };

                use CharClass::*;
                if last_char == Number && digit_right_align {
                    digits = alignment(i);
                }

                while i < range.end {
                    if j >= range.end {
                        if matches!(last_char, Symbol(_)) {
                            zebra_word_boundary(i..j, &mut highlight_scopes);
                        } else {
                            zebra_word(i..j, &mut even, &mut highlight_scopes);
                        }
                        break;
                    }

                    let char = char_at(j);
                    if dyn_align {
                        match char {
                            Symbol('.') => digit_right_align = false,
                            Number | Symbol('_') => (),
                            _ => digit_right_align = true,
                        }
                    }
                    if char == Number {
                        digits += 1;
                        if last_char != Number {
                            digits = if digit_right_align { alignment(j) } else { 0 };
                        }
                    } else {
                        digits = 0;
                    }
                    'r#match: {
                        match (last_char, char) {
                            (Cap, Lower) => {
                                if j < range.start + Ix::new(2) || char_at(j - Ix::new(2)) != Cap {
                                    break 'r#match;
                                }
                                zebra_word(i..j-Ix::new(1), &mut even, &mut highlight_scopes);
                                i = j - Ix::new(1);
                            }
                            (Symbol(_), _) => {
                                if last_char != char {
                                    zebra_word_boundary(i..j, &mut highlight_scopes);
                                    i = j;
                                }
                            }
                            _ => {
                                if last_char != char || last_char == Number && digits >= digit_limit {
                                    zebra_word(i..j, &mut even, &mut highlight_scopes);
                                    i = j;
                                    digits = 0;
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
            let start = self.text().line_of_byte(range.start);
            let end = self.text().pos_of_byte_pos(range.end);
            if let Some(start) = start && let Some(end) = end
                && !(end.line == start || end.column == ix(0) && end.line.saturating_sub(ix(1)) == start) {
                    continue
                }
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
