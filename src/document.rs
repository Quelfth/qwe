use std::iter;

use thiserror::Error;

use crate::rope::{Rope, RopeSlice};

use crate::pos::Pos;

#[derive(Default)]
pub struct Document {
    pub scroll: usize,
    text: Rope,
}

impl Document {
    pub fn new(text: impl AsRef<str>) -> Self {
        Self {
            scroll: 0,
            text: text.as_ref().into(),
        }
    }
}

#[derive(Clone)]
pub struct Change {
    pub byte_pos: usize,
    pub delete: usize,
    pub insert: String,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CursorChangeKind {
    Insert,
    Delete,
}

#[derive(Copy, Clone)]
pub struct CursorChange {
    pub pos: Pos,
    pub kind: CursorChangeKind,
    pub lines: usize,
    pub columns: usize,
}

impl CursorChange {
    pub fn apply(self, pos: Pos) -> Pos {
        let Self {
            pos: change_pos,
            kind,
            lines,
            columns: bytes,
        } = self;
        if pos < change_pos {
            return pos;
        }

        if kind == CursorChangeKind::Insert {
            return Pos {
                line: pos.line + lines,
                column: if lines == 0 { pos.column } else { 0 } + bytes,
            };
        }

        let end_pos = Pos {
            line: change_pos.line + lines,
            column: change_pos.column + bytes,
        };

        if pos > end_pos {
            Pos {
                line: pos.line - lines,
                column: pos.column - bytes,
            }
        } else {
            change_pos
        }
    }

    fn insert(pos: Pos, text: &str) -> Option<Self> {
        (!text.is_empty()).then(|| CursorChange {
            pos,
            kind: CursorChangeKind::Insert,
            lines: text.chars().filter(|&c| c == '\n').count(),
            columns: if !text.ends_with("\n")
                && let Some(line) = text.lines().next_back()
            {
                line.len()
            } else {
                0
            },
        })
    }
}

#[derive(Debug, Error)]
pub enum PosError {
    #[error("line was out of bounds, len was {len}")]
    BadLine { len: usize },
    #[error("column was out of bounds, len was {len}")]
    BadColumn { byte_of_line: usize, len: usize },
}

impl Document {
    pub fn text(&self) -> &Rope {
        &self.text
    }

    pub fn lines_to(&self, height: usize) -> impl Iterator<Item = RopeSlice<'_>> {
        self.text().lines().skip(self.scroll).take(height)
    }

    pub fn byte_pos_of_pos(&self, pos: Pos) -> Result<usize, PosError> {
        if pos.line >= self.text.line_len() {
            return Err(PosError::BadLine {
                len: self.text.line_count(),
            });
        }
        let line = self.text.byte_of_line(pos.line).ok_or(PosError::BadLine {
            len: self.text.line_count(),
        })?;
        let line_len = self
            .text
            .line(pos.line)
            .map(|line| line.byte_len())
            .unwrap_or(0);
        if pos.column > line_len {
            Err::<!, _>(PosError::BadColumn {
                byte_of_line: line,
                len: line_len,
            })?;
        } else {
            Ok(line + pos.column)
        }
    }

    pub fn indent_on_line(&self, line: usize) -> usize {
        let Some(line) = self.text.line(line) else {
            return 0;
        };
        line.graphemes()
            .take_while(|g| g.is_whitespace())
            .map(|g| g.columns())
            .sum()
    }

    pub fn columns_in_line(&self, line: usize) -> usize {
        let Some(line) = self.text.line(line) else {
            return 0;
        };
        line.graphemes().map(|g| g.columns()).sum()
    }

    // pub fn pos_of_byte_pos(&self, byte_pos: usize) -> Option<Pos> {
    //     let line = self.text.line_of_byte(byte_pos)?;
    //     let line_byte = self.text.byte_of_line(line)?;
    //     let column = byte_pos - line_byte;
    //     Some(Pos { line, column })
    // }

    pub fn backspace_change(&self, pos: Pos) -> (Option<Change>, Option<CursorChange>) {
        let change = self.byte_pos_of_pos(pos).ok().and_then(|byte| {
            let grapheme = self
                .text
                .byte_slice(..byte)
                .unwrap()
                .graphemes()
                .next_back()?;
            let size = grapheme.len();
            Some(Change {
                byte_pos: byte - size,
                delete: size,
                insert: "".to_owned(),
            })
        });

        (
            change,
            match pos {
                Pos { line: 0, column: 0 } => None,
                Pos { column: 0, .. } => Some(CursorChange {
                    pos: Pos {
                        line: pos.line - 1,
                        column: self
                            .text
                            .line(pos.line - 1)
                            .map(|l| l.graphemes().count())
                            .unwrap_or(0),
                    },
                    kind: CursorChangeKind::Delete,
                    lines: 1,
                    columns: 0,
                }),
                _ => Some(CursorChange {
                    pos: Pos {
                        line: pos.line,
                        column: pos.column - 1,
                    },
                    kind: CursorChangeKind::Delete,
                    lines: 0,
                    columns: 1,
                }),
            },
        )
    }

    pub fn insert_change(&self, pos: Pos, text: String) -> (Option<Change>, Option<CursorChange>) {
        let cursor_change = CursorChange::insert(pos, &text);
        (
            Some(match self.byte_pos_of_pos(pos) {
                Ok(byte_pos) => Change {
                    byte_pos,
                    delete: 0,
                    insert: text,
                },
                Err(e) => match e {
                    PosError::BadLine { len } => Change {
                        byte_pos: self.text.byte_len(),
                        delete: 0,
                        insert: iter::repeat_n("\n", pos.line - len)
                            .chain(iter::repeat_n(" ", pos.column))
                            .chain(iter::once(&*text))
                            .collect(),
                    },
                    PosError::BadColumn { byte_of_line, len } => Change {
                        byte_pos: byte_of_line + len,
                        delete: 0,
                        insert: iter::repeat_n(" ", pos.column - len)
                            .chain(iter::once(&*text))
                            .collect(),
                    },
                },
            }),
            cursor_change,
        )
    }

    pub fn return_change(&self, pos: Pos) -> (Option<Change>, Option<CursorChange>) {
        (
            Some(Change {
                byte_pos: match self.byte_pos_of_pos(pos) {
                    Ok(pos) => pos,
                    Err(e) => match e {
                        PosError::BadLine { .. } => self.text.byte_len(),
                        PosError::BadColumn { byte_of_line, len } => byte_of_line + len,
                    },
                },
                delete: 0,
                insert: "\n".to_owned(),
            }),
            CursorChange::insert(pos, "\n"),
        )
    }

    pub fn change(
        &mut self,
        Change {
            byte_pos,
            delete,
            insert,
        }: Change,
    ) -> Change {
        let delete_range = byte_pos..byte_pos + delete;
        let deleted = self
            .text
            .byte_slice(delete_range.clone())
            .unwrap()
            .to_string();
        self.text.delete(delete_range).unwrap();
        self.text.insert(byte_pos, &insert).unwrap();
        Change {
            byte_pos,
            delete: insert.len(),
            insert: deleted,
        }
    }

    // pub fn cursor_change(&self, change: &Change) -> Option<CursorChange> {
    //     let Change {
    //         byte_pos,
    //         delete,
    //         insert,
    //     } = change;
    //     let ins = insert;
    //     let insert = insert.len();
    //     match insert.cmp(delete) {
    //         Ordering::Less => {
    //             let byte_pos = byte_pos + insert;
    //             let delete = delete - insert;
    //             let pos = self.pos_of_byte_pos(byte_pos).unwrap();
    //             let end_pos = self.pos_of_byte_pos(byte_pos + delete).unwrap();
    //             let (lines, bytes) = if end_pos.line == pos.line {
    //                 (0, delete)
    //             } else {
    //                 (
    //                     end_pos.line - pos.line,
    //                     byte_pos + delete - self.text.byte_of_line(end_pos.line).unwrap(),
    //                 )
    //             };
    //             Some(CursorChange {
    //                 pos,
    //                 kind: CursorChangeKind::Delete,
    //                 lines,
    //                 columns: bytes,
    //             })
    //         }
    //         Ordering::Equal => None,
    //         Ordering::Greater => Some(CursorChange {
    //             pos: self.pos_of_byte_pos(byte_pos + delete).unwrap(),
    //             kind: CursorChangeKind::Insert,
    //             lines: ins.chars().filter(|&c| c == '\n').count(),
    //             columns: if !ins.ends_with('\n')
    //                 && let Some(line) = ins.lines().next_back()
    //             {
    //                 line.len()
    //             } else {
    //                 0
    //             },
    //         }),
    //     }
    // }
}
