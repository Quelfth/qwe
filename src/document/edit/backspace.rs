use crate::{
    constants::TAB_WIDTH, document::{
        Change,
        CursorChange,
        CursorChangeKind,
        Document,
        PosError,
    }, grapheme::Grapheme, ix::{Byte, Ix, ix}, pos::Pos, rope::Rope, util::{auto_removal_char, indent_string, is_strict_right_delimiter},
};

#[derive(Copy, Clone)]
struct BackspaceContext {
    no_content_before: bool,
    in_indent: bool,
}

fn backspace_context(text: &Rope, pos: Pos) -> BackspaceContext {
    BackspaceContext {
        no_content_before: text.line(pos.line).is_none_or(|l| {
            l.column_slice(..pos.column)
                .chars()
                .all(char::is_whitespace)
        }),
        in_indent: pos.column <= text.context_indent(pos.line),
    }
}

fn text_change(text: &Rope, cx: BackspaceContext, pos: Pos, mirror: bool) -> Option<Change> {
    let BackspaceContext { no_content_before, in_indent } = cx;

    let byte = text
        .byte_pos_of_pos(pos)
        .map(Some)
        .unwrap_or_else(|e| match e {
            PosError::BadLine { .. } => None,
            PosError::BadColumn {
                byte_of_line,
                bytes_in_line,
                ..
            } => (in_indent && no_content_before).then_some(byte_of_line + bytes_in_line),
        })?;

    let mut graphemes = text.byte_slice(..byte).unwrap().graphemes();
    let grapheme = graphemes.next_back()?;
    let (size, extra) = if grapheme.is_whitespace() {
        whitespace_size(text, cx, pos, byte, graphemes, grapheme)
    } else if !mirror && is_strict_right_delimiter(grapheme.as_str()) {
        (ix(0), ix(0)) //strict right delimiters should be backspaced over without being removed
    } else {
        let mut extra = ix(0);
        if !mirror && let Some(char) = auto_removal_char(grapheme.as_str()) {
            let rest = text.byte_slice(byte..).unwrap().graphemes();
            for g in rest {
                let is_char = g.as_str() == char;
                if !(is_char || g.is_whitespace()) {
                    extra = ix(0);
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
        && text.line_has_content(pos.line)
        && let Some(prev_line) = pos.line.checked_sub(ix(1))
        && !text.line_has_content(prev_line)
        && let Some(new_ws) = pos.column.checked_sub(text.columns_in_line(prev_line))
    {
        indent_string(new_ws)
    } else { String::new() };

    Some(Change {
        byte_pos: byte - size,
        delete: size + extra,
        insert: new_whitespace,
    })
}

fn whitespace_size(
    text: &Rope,
    cx: BackspaceContext,
    pos: Pos,
    byte: Ix<Byte>,
    mut graphemes: impl DoubleEndedIterator<Item = Grapheme>,
    grapheme: Grapheme
) -> (Ix<Byte>, Ix<Byte>) {
    let BackspaceContext { no_content_before, in_indent } = cx;
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
        let mut extra = ix(0);
        if let Some(r_delim) = graphemes.next_back().and_then(|g| auto_removal_char(g.as_str())) {
            let mut rest = text.byte_slice(byte..).unwrap().graphemes();
            if rest.next().is_some_and(|n| n.is_newline()) {
                for g in rest {
                    extra += g.len();
                    if g.as_str() == r_delim { break }
                    if !g.is_whitespace() {
                        extra = ix(0);
                        break
                    }
                }
            }
        }
        (sum, extra)
    } else if no_content_before {
        let to_remove = {
            let rem = pos.column % TAB_WIDTH;
            if rem == ix(0) {
                ix(TAB_WIDTH)
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
            ix(0),
        )
    } else {
        (grapheme.len(), ix(0))
    }
}

fn cursor_change(text: &Rope, cx: BackspaceContext, pos: Pos) -> Option<CursorChange> {
    let BackspaceContext { no_content_before, in_indent } = cx;

    if pos == Pos::ZERO { None? }

    if no_content_before {
        if in_indent {
            let line = pos.line - ix(1);
            let column = text.context_columns_in_line(line);
            Some(CursorChange {
                pos: Pos { line, column },
                kind: CursorChangeKind::Delete,
                lines: ix(1),
                columns: pos.column.saturating_sub(column),
            })
        } else {
            let amount = {
                let rem = pos.column % TAB_WIDTH;
                if rem == ix(0) {
                    ix(TAB_WIDTH)
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
                lines: ix(0),
                columns: amount,
            })
        }
    } else {
        Some(CursorChange {
            pos: Pos {
                line: pos.line,
                column: pos.column - ix(1),
            },
            kind: CursorChangeKind::Delete,
            lines: ix(0),
            columns: ix(1),
        })
    }

}

impl Document {
    pub fn backspace_change(&self, pos: Pos, mirror: bool) -> (Option<Change>, Option<CursorChange>) {
        let cx = backspace_context(&self.text, pos);
        (
            text_change(&self.text, cx, pos, mirror),
            cursor_change(&self.text, cx, pos),
        )
    }

    pub fn reverse_backspace_change(&self, pos: Pos) -> (Option<Change>, Option<CursorChange>) {
        let change = self.text.byte_pos_of_pos(pos).ok().and_then(|byte| {
            let grapheme = self.text.byte_slice(byte..).unwrap().graphemes().next()?;
            let size = grapheme.len();
            Some(Change {
                byte_pos: byte,
                delete: size,
                insert: "".to_owned(),
            })
        });

        (
            change,
            Some({
                let (lines, columns) = if pos.column >= self.text.columns_in_line(pos.line) {
                    (ix(1), ix(0))
                } else {
                    (ix(0), ix(1))
                };
                CursorChange {
                    pos,
                    kind: CursorChangeKind::Delete,
                    lines,
                    columns,
                }
            }),
        )
    }
}
