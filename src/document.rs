use std::iter;
use std::ops::Range;
use std::time::Instant;

use thiserror::Error;
use tree_sitter::{InputEdit, Tree};

use crate::aprintln::aprintln;
use crate::constants::TAB_WIDTH;
use crate::document::diagnostics::{Diagnostic, Severity};
use crate::document::history::History;
use crate::document::lsp_change::LspChange;
use crate::document::semtoks::SemanticToken;
use crate::draw::Rect;
use crate::editor::cursors::mirror_insert::InsertDirection;
use crate::editor::cursors::select::{SelectCursor, SelectCursors};
use crate::editor::cursors::{CursorIndex, CursorState};
use crate::grapheme::GraphemeExt;
use crate::ix::{Byte, Column, Ix, Line};
use crate::lang::Language;
use crate::range_sequence::RangeSequence;
use crate::rope::{Rope, RopeSlice};

use crate::pos::{Pos, Region};
use crate::ts::parse_doc;
use crate::util::indent_string;

mod actions;
pub mod diagnostics;
mod find;
mod history;
mod lsp_change;
pub mod semtoks;
mod unopened;

#[derive(Default)]
pub struct Document {
    pub scroll: Ix<Line>,
    pub cursors: Option<CursorState>,
    text: Rope,
    pub history: History,
    pub future: History,
    language: Option<Language>,
    tree: Option<Tree>,
    pub semtoks: RangeSequence<Ix<Byte>, SemanticToken>,
    pub diagnostics: Vec<(Range<Ix<Byte>>, Diagnostic)>,
    pub lsp_version: i32,
    pub lsp_changes: Vec<LspChange>,
    save_prime_instant: Option<Instant>,
}

impl Document {
    pub fn new(
        lang: Option<Language>,
        text: impl AsRef<str>,
        cursors: Option<CursorState>,
    ) -> Self {
        let text: Rope = text.as_ref().into();
        Self {
            tree: lang.map(|lang| parse_doc(&text, None, lang).unwrap()),
            language: lang,
            scroll: Ix::new(0),
            history: Default::default(),
            future: Default::default(),
            semtoks: Default::default(),
            diagnostics: Default::default(),
            cursors,
            text,
            lsp_changes: Vec::new(),
            lsp_version: 0,
            save_prime_instant: None,
        }
    }

    #[allow(unused)]
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
        let n = self.text.max_line_number().inner();
        if n == 0 {
            return 0;
        }
        n.ilog10() as u16 + 1
    }

    pub fn overlay_rect(&self, mut rect: Rect<u16>) -> Rect<u16> {
        rect.cols.start += self.gutter_width();
        rect
    }

    pub fn new_scrolled_cursors(&self) -> impl Fn() -> CursorState + use<> {
        let line = self.scroll;
        move || {
            CursorState::Select(SelectCursors::one(SelectCursor::one_pos(Pos {
                line,
                column: Ix::new(0),
            })))
        }
    }

    pub fn last_line_diagnostic(&self, line: Ix<Line>) -> Option<(Severity, &str)> {
        let range = self.text.byte_range_of_line(line)?;
        let mut diag = None::<(Range<Ix<Byte>>, Severity, &str)>;
        for (r, d) in &self.diagnostics {
            if range.contains(&r.end) && diag.as_ref().is_none_or(|d| r.end >= d.0.end) {
                diag = Some((r.clone(), d.severity, &d.message));
            }
        }
        let (_, s, m) = diag?;
        Some((s, m))
    }
}

macro_rules! force_cursors {
    ($doc: ident) => {{
        let new = $doc.new_scrolled_cursors();
        $doc.cursors.get_or_insert_with(new)
    }};
}
pub(crate) use force_cursors;

#[derive(Clone, Debug)]
pub struct Change {
    pub byte_pos: Ix<Byte>,
    pub delete: Ix<Byte>,
    pub insert: String,
}

impl Change {
    pub fn delete(byte_pos: Ix<Byte>, amount: Ix<Byte>) -> Self {
        Self {
            byte_pos,
            delete: amount,
            insert: "".to_string(),
        }
    }
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
    pub lines: Ix<Line>,
    pub columns: Ix<Column>,
}

pub enum CursorChangeBias {
    Left,
    Right,
}

impl CursorChange {
    pub fn apply(self, pos: Pos, bias: CursorChangeBias) -> Pos {
        use CursorChangeBias::*;
        let Self {
            pos: change_pos,
            kind,
            lines,
            columns,
        } = self;
        if match bias {
            Left => pos <= change_pos,
            Right => pos < change_pos,
        } {
            return pos;
        }

        if kind == CursorChangeKind::Insert {
            return if pos.line == change_pos.line {
                Pos {
                    line: pos.line + lines,
                    column: if lines == Ix::new(0) {
                        pos.column
                    } else {
                        Ix::new(0)
                    } + columns,
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

    pub fn apply_to_line(self, line: Ix<Line>) -> Ix<Line> {
        let Self {
            pos: change_pos,
            kind,
            lines,
            ..
        } = self;
        let change_line = change_pos.line + Ix::new((change_pos.column != Ix::new(0)) as usize);
        if line < change_line {
            return line;
        }

        if kind == CursorChangeKind::Insert {
            return line + lines;
        }

        let end_line = change_line + lines;

        if line > end_line {
            line - lines
        } else {
            change_line
        }
    }

    fn insert(pos: Pos, text: &str) -> Option<Self> {
        (!text.is_empty()).then(|| CursorChange {
            pos,
            kind: CursorChangeKind::Insert,
            lines: Ix::new(text.chars().filter(|&c| c == '\n').count()),
            columns: if !text.ends_with("\n")
                && let Some(line) = text.lines().next_back()
            {
                line.graphemes().map(|g| g.columns()).sum()
            } else {
                Ix::new(0)
            },
        })
    }
}

#[derive(Debug, Error)]
pub enum PosError {
    #[error("line was out of bounds, len was {len:?}")]
    BadLine { len: Ix<Line> },
    #[error("column was out of bounds, len was {bytes_in_line:?}")]
    BadColumn {
        byte_of_line: Ix<Byte>,
        bytes_in_line: Ix<Byte>,
        columns_in_line: Ix<Column>,
    },
}

impl Document {
    pub fn text(&self) -> &Rope {
        &self.text
    }

    pub fn lines_to(&self, height: Ix<Line>) -> impl Iterator<Item = RopeSlice<'_>> {
        self.text()
            .lines()
            .skip(self.scroll.inner())
            .take(height.inner())
    }

    pub fn tab_out_change(&self, pos: Pos) -> (Option<Change>, Option<CursorChange>) {
        if pos.column == Ix::new(0)
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
                    if rem == Ix::new(0) {
                        Ix::new(TAB_WIDTH)
                    } else {
                        rem
                    }
                };
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
        })
    }

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
            let size = if grapheme.is_whitespace() {
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
                    sum
                } else if no_content_before {
                    let to_remove = {
                        let rem = pos.column % TAB_WIDTH;
                        if rem == Ix::new(0) {
                            Ix::new(TAB_WIDTH)
                        } else {
                            rem
                        }
                    };
                    grapheme.len()
                        + graphemes
                            .rev()
                            .take(to_remove.inner() - 1)
                            .map(|g| g.len())
                            .sum()
                } else {
                    grapheme.len()
                }
            } else {
                grapheme.len()
            };
            Some(Change {
                byte_pos: byte - size,
                delete: size,
                insert: "".to_owned(),
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
                    (Ix::new(1), Ix::new(0))
                } else {
                    (Ix::new(0), Ix::new(1))
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

    pub fn insert_change(&self, pos: Pos, text: String) -> (Option<Change>, Option<CursorChange>) {
        let cursor_change = CursorChange::insert(pos, &text);
        (
            Some(match self.text.byte_pos_of_pos(pos) {
                Ok(byte_pos) => Change {
                    byte_pos,
                    delete: Ix::new(0),
                    insert: text,
                },
                Err(e) => match e {
                    PosError::BadLine { len } => Change {
                        byte_pos: self.text.byte_len(),
                        delete: Ix::new(0),
                        insert: iter::repeat_n("\n", (pos.line - len).inner())
                            .chain(iter::repeat_n(" ", (pos.column).inner()))
                            .chain(iter::once(&*text))
                            .collect(),
                    },
                    PosError::BadColumn {
                        byte_of_line,
                        bytes_in_line: len,
                        columns_in_line,
                    } => Change {
                        byte_pos: byte_of_line + len,
                        delete: Ix::new(0),
                        insert: iter::repeat_n(" ", (pos.column - columns_in_line).inner())
                            .chain(iter::once(&*text))
                            .collect(),
                    },
                },
            }),
            cursor_change,
        )
    }

    pub fn return_change(&self, pos: Pos) -> (Option<Change>, Option<CursorChange>) {
        let indent = self.text.context_indent_inc(pos.line);
        let lf_indent = format!("\n{}", indent_string(indent));
        let byte_pos = match self.text.byte_pos_of_pos(pos) {
            Ok(pos) => pos,
            Err(e) => match e {
                PosError::BadLine { .. } => self.text.byte_len(),
                PosError::BadColumn {
                    byte_of_line,
                    bytes_in_line: len,
                    ..
                } => byte_of_line + len,
            },
        };
        (
            Some(Change {
                byte_pos,
                delete: Ix::new(0),
                insert: if self.text.line(pos.line).is_none_or(|l| {
                    l.column_slice(pos.column..)
                        .chars()
                        .all(char::is_whitespace)
                }) {
                    "\n".to_owned()
                } else {
                    lf_indent.clone()
                },
            }),
            CursorChange::insert(pos, &lf_indent),
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
        self.upkeep_delete(delete_range.clone());
        self.text.delete(delete_range).unwrap();
        self.text.insert(byte_pos, &insert).unwrap();
        let insert_len = Ix::new(insert.len());
        self.upkeep_insert(byte_pos, insert);
        if let Some(lang) = self.language {
            self.tree = Some(parse_doc(&self.text, self.tree(), lang).unwrap());
        }
        Change {
            byte_pos,
            delete: insert_len,
            insert: deleted,
        }
    }

    fn upkeep_delete(&mut self, range: Range<Ix<Byte>>) {
        self.tree_delete(range.clone());
        self.lsp_delete(range.clone());
        self.semtoks
            .edit_delete(range.start, range.end - range.start);
    }

    fn upkeep_insert(&mut self, pos: Ix<Byte>, text: String) {
        let len = Ix::new(text.len());
        self.semtoks.edit_insert(pos, len);
        self.lsp_insert(pos, text);
        self.tree_insert(pos, len);
    }

    fn tree_delete(&mut self, range: Range<Ix<Byte>>) {
        if let Some(tree) = &mut self.tree {
            let start = self.text.ts_pos_of_byte(range.start).unwrap();
            let end = self.text.ts_pos_of_byte(range.end).unwrap();
            tree.edit(&InputEdit {
                start_byte: range.start.inner(),
                old_end_byte: range.end.inner(),
                new_end_byte: range.start.inner(),
                start_position: start,
                old_end_position: end,
                new_end_position: start,
            })
        }
    }

    fn tree_insert(&mut self, pos: Ix<Byte>, len: Ix<Byte>) {
        if let Some(tree) = &mut self.tree {
            let start = self.text.ts_pos_of_byte(pos).unwrap();
            let end = self.text.ts_pos_of_byte(pos + len).unwrap();
            tree.edit(&InputEdit {
                start_byte: pos.inner(),
                old_end_byte: pos.inner(),
                new_end_byte: (pos + len).inner(),
                start_position: start,
                old_end_position: start,
                new_end_position: end,
            })
        }
    }

    fn lsp_delete(&mut self, range: Range<Ix<Byte>>) {
        let start = self.text.utf16_pos_of_byte(range.start).unwrap();
        let end = self.text.utf16_pos_of_byte(range.end).unwrap();

        self.lsp_changes.push(LspChange {
            start,
            end,
            text: String::new(),
        })
    }

    fn lsp_insert(&mut self, pos: Ix<Byte>, text: String) {
        let pos = self.text.utf16_pos_of_byte(pos).unwrap();
        self.lsp_changes.push(LspChange {
            start: pos,
            end: pos,
            text,
        })
    }

    pub fn do_insert(
        &mut self,
        change: impl Fn(&Document, Pos, InsertDirection) -> (Option<Change>, Option<CursorChange>),
    ) {
        let Some(cursors) = &self.cursors else { return };
        for i in cursors.indices() {
            self.do_insert_at_index(i, &change);
        }
    }

    pub fn do_insert_at_index(
        &mut self,
        index: CursorIndex,
        change: impl Fn(&Document, Pos, InsertDirection) -> (Option<Change>, Option<CursorChange>),
    ) {
        let Some(cursors) = &self.cursors else { return };
        match cursors {
            CursorState::MirrorInsert(_) => {
                let forward = self.cursors.as_ref().unwrap().assume_mirror_insert()[index].forward;
                self.do_change(change(self, forward, InsertDirection::Forward));
                let reverse = self.cursors.as_ref().unwrap().assume_mirror_insert()[index].reverse;
                self.do_change(change(self, reverse, InsertDirection::Reverse));
            }
            CursorState::Insert(_) => {
                let cursor = self.cursors.as_ref().unwrap().assume_insert()[index];
                self.do_change(change(self, cursor.pos, InsertDirection::Forward))
            }
            _ => todo!(),
        }
    }

    pub fn do_change(&mut self, change: (Option<Change>, Option<CursorChange>)) {
        let (change, cursor_change) = change;
        if let Some(change) = cursor_change
            && let Some(cursors) = &mut self.cursors
        {
            cursors.apply_change(change, &self.text);
        }
        if let Some(change) = change {
            let reverse = self.change(change.clone());
            self.history.push(reverse);
        }
    }

    pub fn undo(&mut self) {
        let mut changes = Vec::<CursorChange>::new();

        self.future.checkpoint();
        for change in self.history.pop().collect::<Vec<_>>().into_iter() {
            if let Some(change) = self.text.cursor_change(&change) {
                changes.push(change);
            }
            let reverse = self.change(change);
            self.future.push(reverse);
        }

        if let Some(cursors) = &mut self.cursors {
            for change in changes {
                cursors.apply_change(change, &self.text);
            }
        }
    }

    pub fn redo(&mut self) {
        let mut changes = Vec::<CursorChange>::new();

        self.history.checkpoint();
        for change in self.future.pop().collect::<Vec<_>>().into_iter() {
            if let Some(change) = self.text.cursor_change(&change) {
                changes.push(change);
            }
            let reverse = self.change(change);
            self.history.push(reverse);
        }

        if let Some(cursors) = &mut self.cursors {
            for change in changes {
                cursors.apply_change(change, &self.text);
            }
        }
    }

    pub fn do_delete(&mut self) {
        self.history.checkpoint();
        if let Some(cursors) = &self.cursors {
            let mut ranges = cursors.delete_ranges(&self.text).collect::<Vec<_>>();
            ranges.sort_unstable_by_key(|r| r.start);
            for range in ranges.into_iter().rev() {
                self.delete(range);
            }
        }
    }

    pub fn delete(&mut self, range: Range<Ix<Byte>>) {
        if range.is_empty() {
            return;
        }
        let change = Change::delete(range.start, range.end - range.start);
        let cursor_change = self.text.cursor_change(&change);
        let reverse = self.change(change);
        self.history.push(reverse);

        if let Some(cursors) = &mut self.cursors
            && let Some(change) = cursor_change
        {
            cursors.apply_change(change, &self.text);
        }
    }

    pub fn inspect_range(&self) -> (Pos, Pos) {
        let Some(cursors) = &self.cursors else {
            return (
                Pos::ZERO,
                Pos {
                    line: self.text.line_len(),
                    column: self.text.columns_in_line(self.text.line_len()),
                },
            );
        };

        match cursors.inspect_range() {
            Region::Pos(range) => (range.start, range.end),
            Region::Line(range) => (
                Pos {
                    line: range.start,
                    column: Ix::new(0),
                },
                Pos {
                    line: range.end,
                    column: self.text.columns_in_line(range.end),
                },
            ),
        }
    }
}
