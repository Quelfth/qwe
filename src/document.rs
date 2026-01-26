use std::cmp::Ordering::*;
use std::iter;
use std::ops::Range;

use thiserror::Error;
use tree_sitter::{InputEdit, Tree};

use crate::aprintln::aprintln;
use crate::document::history::History;
use crate::lang::Language;
use crate::rope::{Rope, RopeSlice};

use crate::pos::Pos;
use crate::ts::parse_doc;

mod history;

#[derive(Default)]
pub struct Document {
    pub scroll: usize,
    text: Rope,
    pub history: History,
    pub future: History,
    language: Option<Language>,
    tree: Option<Tree>,
}

impl Document {
    pub fn new(lang: Option<Language>, text: impl AsRef<str>) -> Self {
        let text: Rope = text.as_ref().into();
        Self {
            tree: lang.map(|lang| parse_doc(&text, None, lang).unwrap()),
            language: lang,
            scroll: 0,
            history: Default::default(),
            future: Default::default(),
            text,
        }
    }

    pub fn print_tree(&self) {
        if let Some(tree) = &self.tree {
            aprintln!("{}", tree.root_node().to_sexp());
        }
    }

    pub fn tree(&self) -> Option<&Tree> {
        self.tree.as_ref()
    }

    pub fn language(&self) -> Option<Language> {
        self.language
    }

    pub fn gutter_width(&self) -> u16 {
        (self.text.line_count() + 1).ilog10() as u16 + 1
    }
}

#[derive(Clone, Debug)]
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
            columns,
        } = self;
        if pos < change_pos {
            return pos;
        }

        if kind == CursorChangeKind::Insert {
            return if pos.line == change_pos.line {
                Pos {
                    line: pos.line + lines,
                    column: if lines == 0 { pos.column } else { 0 } + columns,
                }
            } else {
                Pos {
                    line: pos.line + lines,
                    ..pos
                }
            };
        }

        let end_pos = Pos {
            line: change_pos.line + lines,
            column: change_pos.column + columns,
        };

        if pos > end_pos {
            if pos.line == end_pos.line {
                Pos {
                    line: pos.line - lines,
                    column: pos.column - columns,
                }
            } else {
                Pos {
                    line: pos.line - lines,
                    ..pos
                }
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
        self.tree_delete(delete_range.clone());
        self.text.delete(delete_range).unwrap();
        self.text.insert(byte_pos, &insert).unwrap();
        self.tree_insert(byte_pos, insert.len());
        if let Some(lang) = self.language {
            self.tree = Some(parse_doc(&self.text, self.tree(), lang).unwrap());
        }
        Change {
            byte_pos,
            delete: insert.len(),
            insert: deleted,
        }
    }

    fn tree_delete(&mut self, range: Range<usize>) {
        if let Some(tree) = &mut self.tree {
            let start = self.text.ts_pos_of_byte(range.start).unwrap();
            let end = self.text.ts_pos_of_byte(range.end).unwrap();
            tree.edit(&InputEdit {
                start_byte: range.start,
                old_end_byte: range.end,
                new_end_byte: range.start,
                start_position: start,
                old_end_position: end,
                new_end_position: start,
            })
        }
    }

    fn tree_insert(&mut self, pos: usize, len: usize) {
        if let Some(tree) = &mut self.tree {
            let start = self.text.ts_pos_of_byte(pos).unwrap();
            let end = self.text.ts_pos_of_byte(pos + len).unwrap();
            tree.edit(&InputEdit {
                start_byte: pos,
                old_end_byte: pos,
                new_end_byte: pos + len,
                start_position: start,
                old_end_position: start,
                new_end_position: end,
            })
        }
    }

    pub fn undo(&mut self) -> Vec<CursorChange> {
        let mut changes = Vec::<CursorChange>::new();

        for change in self.history.pop().collect::<Vec<_>>().into_iter() {
            if let Some(change) = self.text.cursor_change(&change) {
                changes.push(change);
            }
            let reverse = self.change(change);
            self.future.push(reverse);
        }
        self.future.checkpoint();

        changes
    }

    pub fn redo(&mut self) -> Vec<CursorChange> {
        let mut changes = Vec::<CursorChange>::new();

        for change in self.future.pop().collect::<Vec<_>>().into_iter() {
            if let Some(change) = self.text.cursor_change(&change) {
                changes.push(change);
            }
            let reverse = self.change(change);
            self.history.push(reverse);
        }
        self.history.checkpoint();

        changes
    }
}
