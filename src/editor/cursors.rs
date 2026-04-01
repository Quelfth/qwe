use std::{
    cmp::Ordering,
    iter, mem,
    ops::{Index, IndexMut, Range},
};

use crate::{
    document::CursorChange,
    editor::cursors::mirror_insert::MirrorInsertCursors,
    ix::{Byte, Column, Ix, Line},
    pos::Region,
    rope::Rope,
};

use auto_enums::auto_enum;
use dispa::dispatch;
use insert::InsertCursors;
use line_select::LineCursors;
use select::SelectCursors;
use tree_sitter::Tree;

pub mod insert;
pub mod line_select;
pub mod mirror_insert;
pub mod select;

#[derive(Copy, Clone)]
pub enum CursorIndex {
    Main,
    Other(usize),
}

#[dispatch(Cursors)]
pub enum CursorState {
    MirrorInsert(MirrorInsertCursors),
    Insert(InsertCursors),
    Select(SelectCursors),
    LineSelect(LineCursors),
}

#[dispatch]
pub trait Cursors {
    fn drop_others(&mut self);

    fn cycle_forward(&mut self);
    fn cycle_backward(&mut self);

    fn collapse_to_start(&mut self);
    fn collapse_to_end(&mut self);

    fn syntax_extend(&mut self, text: &Rope, tree: &tree_sitter::Tree);

    fn line_range_at(&self, index: CursorIndex) -> Range<Ix<Line>>;
    fn line_ranges(&self) -> impl Iterator<Item = Range<Ix<Line>>>;
}

impl<T: Cursor> Cursors for CursorSet<T> {
    fn drop_others(&mut self) {
        self.others.clear();
    }

    fn cycle_forward(&mut self) {
        if self.others.is_empty() {
            return;
        }

        let next = self.others.remove(0);
        self.others.push(mem::replace(&mut self.main, next))
    }

    fn cycle_backward(&mut self) {
        if self.others.is_empty() {
            return;
        }

        let next = self.others.pop().unwrap();
        self.others.insert(0, mem::replace(&mut self.main, next))
    }

    fn collapse_to_start(&mut self) {
        self.iter_mut().for_each(Cursor::collapse_to_start)
    }

    fn collapse_to_end(&mut self) {
        self.iter_mut().for_each(Cursor::collapse_to_end)
    }

    fn syntax_extend(&mut self, text: &Rope, tree: &Tree) {
        self.iter_mut().for_each(|c| c.syntax_extend(text, tree))
    }

    fn line_range_at(&self, index: CursorIndex) -> Range<Ix<Line>> {
        self[index].line_range()
    }

    fn line_ranges(&self) -> impl Iterator<Item = Range<Ix<Line>>> {
        self.iter().map(|c| c.line_range())
    }
}

impl CursorState {
    #[auto_enum(Iterator)]
    pub fn indices(&self) -> impl Iterator<Item = CursorIndex> + use<> {
        use CursorState::*;
        match self {
            MirrorInsert(c) => c.indices(),
            Insert(c) => c.indices(),
            Select(c) => c.indices(),
            LineSelect(c) => c.indices(),
        }
    }

    pub fn assume_mirror_insert(&self) -> &MirrorInsertCursors {
        let Self::MirrorInsert(cursors) = self else {
            panic!()
        };
        cursors
    }
    pub fn assume_insert(&self) -> &InsertCursors {
        let Self::Insert(cursors) = self else {
            panic!()
        };
        cursors
    }
    #[allow(unused)]
    pub fn assume_select(&self) -> &SelectCursors {
        let Self::Select(cursors) = self else {
            panic!()
        };
        cursors
    }
    #[allow(unused)]
    pub fn assume_line_select(&self) -> &LineCursors {
        let Self::LineSelect(cursors) = self else {
            panic!()
        };
        cursors
    }
    #[allow(unused)]
    pub fn assume_mirror_insert_mut(&mut self) -> &mut MirrorInsertCursors {
        let Self::MirrorInsert(cursors) = self else {
            panic!()
        };
        cursors
    }
    pub fn assume_insert_mut(&mut self) -> &mut InsertCursors {
        let Self::Insert(cursors) = self else {
            panic!()
        };
        cursors
    }
    #[allow(unused)]
    pub fn assume_select_mut(&mut self) -> &mut SelectCursors {
        let Self::Select(cursors) = self else {
            panic!()
        };
        cursors
    }
    #[allow(unused)]
    pub fn assume_line_select_mut(&mut self) -> &mut LineCursors {
        let Self::LineSelect(cursors) = self else {
            panic!()
        };
        cursors
    }
}

#[derive(Clone, Default)]
pub struct CursorSet<Cursor> {
    main: Cursor,
    others: Vec<Cursor>,
}

impl<T> CursorSet<T> {
    pub fn main(&self) -> &T {
        &self.main
    }

    pub fn one(cursor: T) -> Self {
        Self {
            main: cursor,
            others: Vec::new(),
        }
    }

    pub fn from_iter(iter: impl IntoIterator<Item = T>) -> Option<Self> {
        let mut iter = iter.into_iter();
        Some(Self {
            main: iter.next()?,
            others: iter.collect(),
        })
    }

    pub fn indices(&self) -> impl Iterator<Item = CursorIndex> + use<T> {
        iter::once(CursorIndex::Main).chain((0..self.others.len()).map(CursorIndex::Other))
    }
}

impl Default for CursorState {
    fn default() -> Self {
        Self::Select(Default::default())
    }
}

impl From<MirrorInsertCursors> for CursorState {
    fn from(value: MirrorInsertCursors) -> Self {
        Self::MirrorInsert(value)
    }
}

impl From<InsertCursors> for CursorState {
    fn from(value: InsertCursors) -> Self {
        Self::Insert(value)
    }
}

impl From<SelectCursors> for CursorState {
    fn from(value: SelectCursors) -> Self {
        Self::Select(value)
    }
}

impl From<LineCursors> for CursorState {
    fn from(value: LineCursors) -> Self {
        Self::LineSelect(value)
    }
}

impl CursorState {
    pub fn apply_change(&mut self, change: CursorChange, text: &Rope) {
        use CursorState::*;
        match self {
            MirrorInsert(cursors) => cursors.apply_change(change, text),
            Insert(cursors) => cursors.apply_change(change, text),
            Select(cursors) => cursors.apply_change(change, text),
            LineSelect(cursors) => cursors.apply_change(change, text),
        }
    }

    pub fn move_x(&mut self, columns: Ix<Column, isize>) {
        use CursorState::*;
        match self {
            MirrorInsert(_) => todo!(),
            Insert(cursors) => cursors.move_x(columns),
            Select(cursors) => cursors.move_x(columns),
            LineSelect(_) => (),
        }
    }

    pub fn move_y(&mut self, rows: Ix<Line, isize>) {
        use CursorState::*;
        match self {
            MirrorInsert(_) => todo!(),
            Insert(c) => c.move_y(rows),
            Select(c) => c.move_y(rows),
            LineSelect(c) => c.move_y(rows),
        }
    }

    pub fn text_extend_up(&mut self, rows: Ix<Line>, text: &Rope) {
        use CursorState::*;
        match self {
            MirrorInsert(_) => (),
            Insert(_) => (),
            Select(c) => c.iter_mut().for_each(|c| c.text_extend_up(rows, text)),
            LineSelect(c) => c.iter_mut().for_each(|c| c.extend_up(rows)),
        }
    }

    pub fn text_extend_down(&mut self, rows: Ix<Line>, text: &Rope) {
        use CursorState::*;
        match self {
            MirrorInsert(_) => todo!(),
            Insert(_) => (),
            Select(c) => c.iter_mut().for_each(|c| c.text_extend_down(rows, text)),
            LineSelect(c) => c.iter_mut().for_each(|c| c.extend_down(rows)),
        }
    }
    pub fn extend_left(&mut self, columns: Ix<Column>) {
        if let CursorState::Select(c) = self {
            c.iter_mut().for_each(|c| c.extend_left(columns))
        }
    }
    pub fn extend_right(&mut self, columns: Ix<Column>) {
        if let CursorState::Select(c) = self {
            c.iter_mut().for_each(|c| c.extend_right(columns))
        }
    }
    pub fn retract_up(&mut self, rows: Ix<Line>) {
        use CursorState::*;
        match self {
            MirrorInsert(_) => (),
            Insert(_) => (),
            Select(c) => c.iter_mut().for_each(|c| c.retract_up(rows)),
            LineSelect(c) => c.iter_mut().for_each(|c| c.retract_up(rows)),
        }
    }

    pub fn retract_down(&mut self, rows: Ix<Line>) {
        use CursorState::*;
        match self {
            MirrorInsert(_) => (),
            Insert(_) => (),
            Select(c) => c.iter_mut().for_each(|c| c.retract_down(rows)),
            LineSelect(c) => c.iter_mut().for_each(|c| c.retract_down(rows)),
        }
    }
    pub fn retract_left(&mut self, rows: Ix<Column>) {
        if let CursorState::Select(c) = self {
            c.iter_mut().for_each(|c| c.retract_left(rows))
        }
    }
    pub fn retract_right(&mut self, rows: Ix<Column>) {
        if let CursorState::Select(c) = self {
            c.iter_mut().for_each(|c| c.retract_right(rows))
        }
    }

    pub fn inspect_range(&self) -> Region {
        use CursorState::*;
        match self {
            MirrorInsert(_) => todo!(),
            Insert(c) => c.main.inspect_range(),
            Select(c) => c.main.inspect_range(),
            LineSelect(c) => c.main.inspect_range(),
        }
    }

    #[auto_enum(Iterator)]
    pub fn delete_ranges(&self, text: &Rope) -> impl Iterator<Item = Range<Ix<Byte>>> {
        use CursorState::*;
        match self {
            MirrorInsert(_) => iter::empty(),
            Insert(_) => iter::empty(),
            Select(c) => c.delete_ranges(text),
            LineSelect(c) => c.delete_ranges(text),
        }
    }
}

impl<T> CursorSet<T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        iter::once(&self.main).chain(&self.others)
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        iter::once(&mut self.main).chain(&mut self.others)
    }

    /// main first, then others in sorted order by location
    pub fn sorted_iter(&self) -> impl Iterator<Item = &T>
    where
        T: Cursor,
    {
        iter::once(&self.main).chain({
            let mut sorted = self.others.iter().collect::<Vec<_>>();
            sorted.sort_unstable_by(|a, b| T::location_cmp(a, b));
            sorted.into_iter()
        })
    }

    pub fn map_to<U>(&self, map: impl Fn(&T) -> U) -> CursorSet<U> {
        CursorSet {
            main: map(&self.main),
            others: self.others.iter().map(map).collect(),
        }
    }

    pub fn apply_change(&mut self, change: CursorChange, text: &Rope)
    where
        T: Cursor,
    {
        for cursor in self.iter_mut() {
            cursor.apply_change(change, text)
        }
    }

    pub fn get(&self, i: CursorIndex) -> Option<&T> {
        match i {
            CursorIndex::Main => Some(&self.main),
            CursorIndex::Other(i) => self.others.get(i),
        }
    }
    #[allow(unused)]
    pub fn get_mut(&mut self, i: CursorIndex) -> Option<&mut T> {
        match i {
            CursorIndex::Main => Some(&mut self.main),
            CursorIndex::Other(i) => self.others.get_mut(i),
        }
    }
}

impl<T> IndexMut<CursorIndex> for CursorSet<T> {
    fn index_mut(&mut self, index: CursorIndex) -> &mut Self::Output {
        match index {
            CursorIndex::Main => &mut self.main,
            CursorIndex::Other(i) => &mut self.others[i],
        }
    }
}

impl<T> Index<CursorIndex> for CursorSet<T> {
    type Output = T;

    fn index(&self, index: CursorIndex) -> &Self::Output {
        match index {
            CursorIndex::Main => &self.main,
            CursorIndex::Other(i) => &self.others[i],
        }
    }
}

pub trait Cursor {
    fn apply_change(&mut self, change: CursorChange, text: &Rope);
    fn location_cmp(left: &Self, right: &Self) -> Ordering;

    fn collapse_to_start(&mut self) {}
    fn collapse_to_end(&mut self) {}

    fn syntax_extend(&mut self, #[expect(unused)] text: &Rope, #[expect(unused)] tree: &Tree) {}

    fn line_range(&self) -> Range<Ix<Line>>;
}