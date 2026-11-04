use crate::{constants::TAB_WIDTH, document::{Change, CursorChange, CursorChangeKind, Document, PosError}, ix::ix, pos::Pos};

impl Document {
    pub fn tab_out_change(&self, pos: Pos) -> (Option<Change>, Option<CursorChange>) {
            if pos.column == ix(0)
                || !self.text.line(pos.line).is_none_or(|l| {
                    l.column_slice(..pos.column)
                        .chars()
                        .all(char::is_whitespace)
                })
            {
                return (None, None);
            }
    
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
                    } => Some(byte_of_line + bytes_in_line),
                });
    
            let change = p.and_then(|byte| {
                let mut graphemes = self.text.byte_slice(..byte).unwrap().graphemes();
                let grapheme = graphemes.next_back()?;
                let size = {
                    let to_remove = {
                        let rem = pos.column % TAB_WIDTH;
                        if rem == ix(0) {
                            ix(TAB_WIDTH)
                        } else {
                            rem
                        }
                    };
                    if grapheme.is_newline() {
                        return None;
                    }
                    grapheme.len()
                        + graphemes
                            .rev()
                            .take(to_remove.inner() - 1)
                            .map(|g| g.len())
                            .sum()
                };
                Some(Change {
                    byte_pos: byte - size,
                    delete: size,
                    insert: "".to_owned(),
                })
            });
    
            (change, {
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
            })
        }    
}