use std::iter;
use std::ops::Range;
use std::time::Instant;

use mutx::Mutex;
use thiserror::Error;
use tree_sitter::{InputEdit, Tree};

use crate::{
    aprintln::aprintln, constants::TAB_WIDTH, document::{
        diagnostics::{Diagnostic, Severity},
        lsp_change::LspChange,
        semtoks::SemanticToken,
    }, draw::Rect, editor::cursors::{
        CursorIndex, CursorState, mirror_insert::InsertDirection, select::{SelectCursor, SelectCursors}
    }, grapheme::GraphemeExt, ix::{Byte, Column, Ix, Line}, lang::Language, pos::{Pos, Region, Utf16Pos}, range_sequence::RangeSequence, rope::{Rope, RopeSlice}, timeline::{
        TimeDirection, Timeline, document::{DocumentEvent, TimeStackPop}, global::GlobalCheckpoint
    }, ts::parse_doc, util::{LinesColumnsExt, flip_delimiter, indent_string, is_right_delimiter}
};

mod actions;
pub mod diagnostics;
mod edit;
mod find;
mod lsp_change;
pub mod semtoks;
mod unopened;

#[derive(Default)]
pub struct Document {
    pub scroll: Ix<Line>,
    pub horizontal_scroll: Ix<Column>,
    pub view_height: Mutex<Ix<Line>>,
    pub cursors: Option<CursorState>,
    text: Rope,
    pub timeline: Timeline<DocumentEvent>,
    language: Option<Language>,
    tree: Option<Tree>,
    pub semtoks: RangeSequence<Ix<Byte>, SemanticToken>,
    pub diagnostics: RangeSequence<Ix<Byte>, Diagnostic>,
    pub lsp_version: i32,
    pub lsp_changes: Vec<LspChange>,
    #[expect(unused)]
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
            horizontal_scroll: Ix::new(0),
            view_height: Default::default(),
            timeline: Default::default(),
            semtoks: Default::default(),
            diagnostics: Default::default(),
            cursors,
            text,
            lsp_changes: Vec::new(),
            lsp_version: 1,
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
        let mut diagnostic = None::<(Range<Ix<Byte>>, Severity, &str)>;
        for (r, d) in self.diagnostics.ranges() {
            if range.start <= r.end
                && range.end >= r.end
                && diagnostic.as_ref().is_none_or(|(range, severity, _)| {
                    r.end >= range.end && *severity <= d.severity
                })
            {
                diagnostic = Some((r.clone(), d.severity, &d.message));
            }
        }
        let (_, s, m) = diagnostic?;
        Some((s, m))
    }

    pub fn main_cursor_line(&self) -> Ix<Line> {
        let Some(cursors) = &self.cursors else {
            return Ix::new(0);
        };
        match cursors {
            CursorState::MirrorInsert(cursors) => cursors.main().forward.line,
            CursorState::Insert(cursors) => cursors.main().pos.line,
            CursorState::Select(cursors) => cursors.main().start_pos().line,
            CursorState::LineSelect(cursors) => cursors.main().line,
        }
    }

    pub fn main_cursor_pos(&self) -> Option<Pos> {
        Some(match self.cursors.as_ref()? {
            CursorState::MirrorInsert(cursors) => cursors.main().forward,
            CursorState::Insert(cursors) => cursors.main().pos,
            CursorState::Select(cursors) => cursors.main().start_pos(),
            CursorState::LineSelect(cursors) => Pos {
                line: cursors.main().line,
                column: Ix::new(0),
            },
        })
    }

    pub fn main_cursor_pos_utf16(&self) -> Option<Utf16Pos> {
        self.main_cursor_pos()
            .and_then(|pos| self.text.utf16_pos_of_pos(pos))
    }

    pub fn main_cursor_is_visible(&self) -> bool {
        (self.scroll..self.scroll + *self.view_height.lock()).contains(&self.main_cursor_line())
    }

    pub fn scroll_main_cursor_on_screen(&mut self) {
        if !self.main_cursor_is_visible() {
            self.scroll_to_main_cursor();
        }
    }

    pub fn screen_line_range(&self) -> Range<Ix<Line>> {
        self.scroll..self.scroll + *self.view_height.lock()
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
    Insert(Pos),
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

        if let CursorChangeKind::Insert(center) = kind {
            return if pos.line == change_pos.line {
                if pos.column == change_pos.column { center } else {
                    Pos {
                        line: pos.line + lines,
                        column: if lines == Ix::new(0) {
                            pos.column
                        } else {
                            Ix::new(0)
                        } + columns,
                    }
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

        if matches!(kind, CursorChangeKind::Insert(_)) {
            return line + lines;
        }

        let end_line = change_line + lines;

        if line > end_line {
            line - lines
        } else {
            change_line
        }
    }

    fn insert(pos: Pos, text: &str, center: Pos) -> Option<Self> {
        (!text.is_empty()).then(|| CursorChange {
            pos,
            kind: CursorChangeKind::Insert(center),
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

    #[expect(unused)]
    fn insert_start(pos: Pos, text: &str) -> Option<Self> {
        Self::insert(pos, text, pos)
    }

    pub fn insert_end(pos: Pos, text: &str) -> Option<Self> {
        if text.is_empty() { return None }
        let (lines, columns) = text.lines_columns();

        Some(CursorChange {
            pos,
            kind: CursorChangeKind::Insert(pos.offset(lines, columns)),
            lines,
            columns,
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

    fn insert_change_inner(&self, pos: Pos, text: String) -> Change {
        match self.text.byte_pos_of_pos(pos) {
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
        }
    }

    pub fn insert_change(&self, pos: Pos, text: String) -> (Option<Change>, Option<CursorChange>) {
        let cursor_change = CursorChange::insert_end(pos, &text);
        (
            Some(self.insert_change_inner(pos, text)),
            cursor_change,
        )
    }

    pub fn insert_pair_change(&self, pos: Pos, left: String, right: String) -> (Option<Change>, Option<CursorChange>) {
        let (lines, columns) = left.lines_columns();
        let total = left + &right;
        let cursor_change = CursorChange::insert(pos, &total, pos.offset(lines, columns));
        (
            Some(self.insert_change_inner(pos, total)),
            cursor_change,
        )
    }

    pub fn insert_reluctant_change(&self, pos: Pos, text: String) -> (Option<Change>, Option<CursorChange>) {
        let cursor_change = CursorChange::insert_end(pos, &text);
        (
            self.text
                .byte_pos_of_pos(pos).ok()
                .is_none_or(|pos|
                    self.text
                        .byte_slice(pos..pos + Ix::new(text.len()))
                        .is_none_or(|slice| slice.to_string() != text)
                )
                .then(|| self.insert_change_inner(pos, text)),
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
        if let Some(g) = self.text.byte_slice(byte_pos..).unwrap().graphemes().next() && !g.is_newline() {
            let g = g.as_str();
            if is_right_delimiter(g) {
                let indent = if let Some(l) = flip_delimiter(g)
                    && let Some(d) = self.text.byte_slice(..byte_pos).unwrap().graphemes().next_back()
                    && d.as_str() == l
                {
                    indent
                } else {
                    indent.saturating_sub(Ix::new(TAB_WIDTH))
                };
                let indent1 = indent + Ix::new(TAB_WIDTH);
                let insert = format!("\n{}\n{}", indent_string(indent1), indent_string(indent));
                return (
                    Some(Change {
                        byte_pos,
                        delete: Ix::new(0),
                        insert,
                    }),
                    CursorChange::insert(pos, &lf_indent, pos.offset(Ix::new(1), indent1)),
                );
            }
        }
        
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
            CursorChange::insert_end(pos, &lf_indent),
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
        let len = range.end - range.start;
        self.semtoks.edit_delete(range.start, len);
        self.diagnostics.edit_delete(range.start, len);
    }

    fn upkeep_insert(&mut self, pos: Ix<Byte>, text: String) {
        let len = Ix::new(text.len());
        self.diagnostics.edit_insert(pos, len);
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
            self.timeline.history.push(reverse);
        }
    }
    
    pub fn unredo(&mut self, direction: TimeDirection) -> Result<(), GlobalCheckpoint> {
        match self.timeline[direction].pop() {
            TimeStackPop::Local(popped) => {
                self.local_unredo(direction, None, popped);
                Ok(())
            },
            TimeStackPop::Global(cp) => {
                Err(cp)
            },
            TimeStackPop::Empty => Ok(()),
        }
    }

    fn local_unredo(&mut self, dir: TimeDirection, global: Option<GlobalCheckpoint>, to_do: Vec<Change>) {
        let mut changes = Vec::<CursorChange>::new();
        
        if global.is_some() {
            self.timeline[dir.rev()].global_checkpoint();
        } else {
            self.timeline[dir.rev()].checkpoint();
        }
        for change in to_do {
            if let Some(change) = self.text.cursor_change(&change) {
                changes.push(change);
            }
            let reverse = self.change(change);
            self.timeline[dir.rev()].push(reverse);
        }
        if let Some(cp) = global {
            self.timeline[dir.rev()].push_global_jump(cp);
        }
    
        if let Some(cursors) = &mut self.cursors {
            for change in changes {
                cursors.apply_change(change, &self.text);
            }
        }
    }

    pub fn global_unredo(&mut self, dir: TimeDirection, cp: GlobalCheckpoint, count: u32) {
        let changes = self.timeline[dir].pop_global(count);
        self.local_unredo(dir, Some(cp), changes);
    }

    pub fn do_delete(&mut self) {
        self.timeline.history.checkpoint();
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
        self.timeline.history.push(reverse);

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
