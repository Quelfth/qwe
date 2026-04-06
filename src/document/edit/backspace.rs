use 
    crate::{
        constants::TAB_WIDTH,
        document::{
            Change,
            CursorChange,
            CursorChangeKind,
            Document,
            PosError,
        },
        ix::Ix,
        pos::Pos,
        util::{auto_removal_char, indent_string},
    }
;

impl Document {
    pub fn backspace_change(&self, pos: Pos) -> (Option<Change>, Option<CursorChange>) {
        let indent = self.text.context_indent(pos.line);
        let no_content_before = self.text.line(pos.line).is_none_or(|l| {
            l.column_slice(..pos.column)
                .chars()
                .all(char::is_whitespace)
        });
        let in_indent = pos.column <= indent;
        let p = self
            .text
            .byte_pos_of_pos(pos)
            .map(Some)
            .unwrap_or_else(|e| match e {
                PosError::BadLine { .. } => None,
                PosError::BadColumn {
                    byte_of_line,
                    bytes_in_line,
                    ..
                } => (in_indent && no_content_before).then_some(byte_of_line + bytes_in_line),
            });
    
        let change = p.and_then(|byte| {
            let mut graphemes = self.text.byte_slice(..byte).unwrap().graphemes();
            let grapheme = graphemes.next_back()?;
            let (size, extra) = if grapheme.is_whitespace() {
                if in_indent {
                    let mut sum = grapheme.len();
                    if !grapheme.is_newline() {
                        while let Some(g) = graphemes.next_back() {
                            if !g.is_whitespace() {
                                sum = grapheme.len();
                                break;
                            }
                            sum += g.len();
                            if g.is_newline() {
                                break;
                            }
                        }
                    }
                    let mut extra = Ix::new(0);
                    if let Some(r_delim) = graphemes.next_back().and_then(|g| auto_removal_char(g.as_str())) {
                        let mut rest = self.text.byte_slice(byte..).unwrap().graphemes();
                        if rest.next().is_some_and(|n| n.is_newline()) {
                            for g in rest {
                                extra += g.len();
                                if g.as_str() == r_delim { break }
                                if !g.is_whitespace() {
                                    extra = Ix::new(0);
                                    break
                                }
                            }
                        }
                    }
                    (sum, extra)
                } else if no_content_before {
                    let to_remove = {
                        let rem = pos.column % TAB_WIDTH;
                        if rem == Ix::new(0) {
                            Ix::new(TAB_WIDTH)
                        } else {
                            rem
                        }
                    };
                    (
                        grapheme.len()
                            + graphemes
                                .rev()
                                .take(to_remove.inner() - 1)
                                .map(|g| g.len())
                                .sum(),
                        Ix::new(0),
                    )
                } else {
                    (grapheme.len(), Ix::new(0))
                }
            } else {
                let mut extra = Ix::new(0);
                if let Some(char) = auto_removal_char(grapheme.as_str()) {
                    let rest = self.text.byte_slice(byte..).unwrap().graphemes();
                    for g in rest {
                        let is_char = g.as_str() == char;
                        if !(is_char || g.is_whitespace()) {
                            extra = Ix::new(0);
                            break;
                        }
                        extra += g.len();
                        if is_char { break }
                    }
                }
                (grapheme.len(), extra)
            };
            let new_whitespace = if in_indent 
                && no_content_before 
                && let Some(prev_line) = pos.line.checked_sub(Ix::new(1))
                && !self.text.line_has_content(prev_line) 
                && let Some(new_ws) = pos.column.checked_sub(self.text.columns_in_line(prev_line)) 
            {
                indent_string(new_ws)
            } else {String::new()};
            Some(Change {
                byte_pos: byte - size,
                delete: size + extra,
                insert: new_whitespace,
            })
        });
    
        (
            change,
            match pos {
                Pos { line, column } if line == Ix::new(0) && column == Ix::new(0) => None,
                _ if no_content_before => {
                    if in_indent {
                        Some(CursorChange {
                            pos: Pos {
                                line: pos.line - Ix::new(1),
                                column: self
                                    .text
                                    .line(pos.line - Ix::new(1))
                                    .map(|l| l.graphemes().map(|g| g.columns()).sum())
                                    .unwrap_or(Ix::new(0)),
                            },
                            kind: CursorChangeKind::Delete,
                            lines: Ix::new(1),
                            columns: Ix::new(0),
                        })
                    } else {
                        let amount = {
                            let rem = pos.column % TAB_WIDTH;
                            if rem == Ix::new(0) {
                                Ix::new(TAB_WIDTH)
                            } else {
                                rem
                            }
                        };
    
                        Some(CursorChange {
                            pos: Pos {
                                line: pos.line,
                                column: pos.column - amount,
                            },
                            kind: CursorChangeKind::Delete,
                            lines: Ix::new(0),
                            columns: amount,
                        })
                    }
                }
                _ => Some(CursorChange {
                    pos: Pos {
                        line: pos.line,
                        column: pos.column - Ix::new(1),
                    },
                    kind: CursorChangeKind::Delete,
                    lines: Ix::new(0),
                    columns: Ix::new(1),
                }),
            },
        )
    }
}