use std::ops::{Range, RangeBounds};

use crop::iter::{Bytes, Chars, Chunks, RawLines};

use crate::{
    aprintln::aprintln,
    document::{Change, CursorChange, CursorChangeKind},
    grapheme::{Grapheme, GraphemeExt},
    ix::{self, Byte, Column, Ix, Line, MappedRange, ixto},
    pos::Pos,
    rope::iter::{Graphemes, Lines},
};

use super::{Rope, RopeSlice, range_bounds_to_start_end};

use std::cmp::Ordering::*;

impl Rope {
    #[must_use]
    fn validate_byte_range(&self, byte_range: &(impl RangeBounds<Ix<Byte>> + Clone)) -> Option<()> {
        let (start, end) =
            range_bounds_to_start_end(byte_range.clone(), Ix::new(0), self.byte_len());
        if start > end {
            return None;
        }
        if end > self.byte_len() {
            return None;
        }
        if !self.0.is_char_boundary(start.inner()) || !self.0.is_char_boundary(end.inner()) {
            return None;
        }

        Some(())
    }

    #[must_use]
    fn validate_byte_offset(&self, byte_offset: Ix<Byte>) -> Option<()> {
        ixto!(byte_offset);
        (byte_offset <= self.0.byte_len() && self.0.is_char_boundary(byte_offset)).then_some(())
    }

    fn validate_line_range(&self, line_range: &(impl RangeBounds<Ix<Line>> + Clone)) -> Option<()> {
        let (start, end) =
            range_bounds_to_start_end(line_range.clone(), Ix::new(0), Ix::new(self.0.line_len()));
        if start > end {
            return None;
        }
        if end > Ix::new(self.0.line_len()) {
            return None;
        }

        Some(())
    }

    pub fn byte(&self, i: Ix<Byte>) -> Option<u8> {
        let i = i.inner();
        (i < self.0.byte_len()).then(|| self.0.byte(i))
    }

    pub fn byte_len(&self) -> Ix<Byte> {
        Ix::new(self.0.byte_len())
    }

    pub fn byte_of_line(&self, line_offset: Ix<Line>) -> Option<Ix<Byte>> {
        ixto!(line_offset);
        (line_offset <= self.0.line_len()).then(|| Ix::new(self.0.byte_of_line(line_offset)))
    }

    pub fn byte_slice(
        &self,
        byte_range: impl RangeBounds<Ix<Byte>> + Clone,
    ) -> Option<RopeSlice<'_>> {
        self.validate_byte_range(&byte_range)?;
        Some(self.0.byte_slice(MappedRange::new(byte_range)).into())
    }

    pub fn bytes(&self) -> Bytes<'_> {
        self.0.bytes()
    }

    pub fn chars(&self) -> Chars<'_> {
        self.0.chars()
    }

    pub fn chunks(&self) -> Chunks<'_> {
        self.0.chunks()
    }

    #[must_use]
    pub fn delete(&mut self, byte_range: impl RangeBounds<Ix<Byte>> + Clone) -> Option<()> {
        self.validate_byte_range(&byte_range)?;
        self.0.delete(MappedRange::new(byte_range));
        Some(())
    }

    pub fn graphemes(&self) -> Graphemes<'_> {
        Graphemes(self.0.graphemes())
    }

    pub fn graphemes_with_bytes(&self) -> impl Iterator<Item = (Ix<Byte>, Grapheme)> {
        let iter = self.graphemes();
        gen {
            let mut byte = Ix::new(0);

            for grapheme in iter {
                let len = grapheme.len();
                yield (byte, grapheme);
                byte += len;
            }
        }
    }

    pub fn columns_bytes(&self) -> impl Iterator<Item = (Ix<Column>, Ix<Byte>)> {
        let iter = self.graphemes();
        gen {
            let mut column = Ix::new(0);
            let mut byte = Ix::new(0);

            for grapheme in iter {
                let columns = grapheme.columns();
                let bytes = grapheme.len();
                for _ in Ix::<Column>::new(0)..columns {
                    yield (column, byte);
                    column += Ix::new(1);
                }
                byte += bytes;
            }
        }
    }

    #[must_use]
    pub fn insert(&mut self, byte_offset: Ix<Byte>, text: impl AsRef<str>) -> Option<()> {
        self.validate_byte_offset(byte_offset)?;
        self.0.insert(byte_offset.inner(), text);
        Some(())
    }

    pub fn is_char_boundary(&self, byte_offset: Ix<Byte>) -> Option<bool> {
        if byte_offset > self.byte_len() {
            return None;
        }
        Some(self.0.is_char_boundary(byte_offset.inner()))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn is_grapheme_boundary(&self, byte_offset: Ix<Byte>) -> Option<bool> {
        if byte_offset > self.byte_len() {
            return None;
        }
        Some(self.0.is_grapheme_boundary(byte_offset.inner()))
    }

    pub fn line(&self, line_index: Ix<Line>) -> Option<RopeSlice<'_>> {
        ixto!(line_index);
        if line_index >= self.0.line_len() {
            return None;
        }
        Some(self.0.line(line_index).into())
    }

    pub fn line_len(&self) -> Ix<Line> {
        Ix::new(self.0.line_len())
    }

    pub fn line_count(&self) -> Ix<Line> {
        let len = self.0.line_len();
        Ix::new(match self.0.chars().next_back() {
            Some('\n') => len,
            None => 0,
            _ => len - 1,
        })
    }

    pub fn line_of_byte(&self, byte_offset: Ix<Byte>) -> Option<Ix<Line>> {
        ixto!(byte_offset);
        if byte_offset > self.0.byte_len() {
            return None;
        }
        Some(Ix::new(self.0.line_of_byte(byte_offset)))
    }

    pub fn line_slice(
        &self,
        line_range: impl RangeBounds<Ix<Line>> + Clone,
    ) -> Option<RopeSlice<'_>> {
        self.validate_line_range(&line_range)?;
        Some(self.0.line_slice(MappedRange::new(line_range)).into())
    }

    pub fn lines(&self) -> Lines<'_> {
        Lines(self.0.lines())
    }

    pub fn raw_lines(&self) -> RawLines<'_> {
        self.0.raw_lines()
    }

    pub fn new() -> Self {
        Self(crop::Rope::new())
    }

    #[must_use]
    pub fn replace(
        &mut self,
        byte_range: impl RangeBounds<Ix<Byte>> + Clone,
        text: impl AsRef<str>,
    ) -> Option<()> {
        self.validate_byte_range(&byte_range)?;
        self.0.replace(MappedRange::new(byte_range), text);
        Some(())
    }

    pub fn chunk_from_byte(&self, byte: Ix<Byte>) -> Option<&str> {
        Some(self.byte_slice(byte..)?.chunks().next().unwrap_or(""))
    }

    pub fn ts_callback<'a>(&'a self) -> impl Fn(usize, tree_sitter::Point) -> &'a str {
        |byte, _| {
            self.chunk_from_byte(Ix::new(byte))
                .expect("tree sitter should be providing valid byte offsets")
        }
    }

    pub fn ts_pos_of_byte(&self, byte: Ix<Byte>) -> Option<tree_sitter::Point> {
        let line = self.line_of_byte(byte)?;
        Some(tree_sitter::Point {
            row: line.inner(),
            column: (byte - self.byte_of_line(line)?).inner(),
        })
    }

    pub fn pos_of_byte_pos(&self, byte_pos: Ix<Byte>) -> Option<Pos> {
        let line = self.line_of_byte(byte_pos)?;
        let line_byte = self.byte_of_line(line)?;
        let byte_in_line = byte_pos - line_byte;
        let Some(line_text) = self.line(line) else {
            return Some(Pos {
                line,
                column: Ix::new(0),
            });
        };
        let column = line_text
            .byte_slice(..byte_in_line)?
            .graphemes()
            .map(|g| g.columns())
            .sum();
        Some(Pos { line, column })
    }

    pub fn cursor_change(&self, change: &Change) -> Option<CursorChange> {
        let Change {
            byte_pos,
            delete,
            insert,
        } = change;
        let ins = insert;
        let insert = Ix::new(insert.len());
        match insert.cmp(delete) {
            Less => {
                let byte_pos = *byte_pos + insert;
                let delete = *delete - insert;
                let pos = self.pos_of_byte_pos(byte_pos).unwrap();
                let end_pos = self.pos_of_byte_pos(byte_pos + delete).unwrap();
                let (lines, columns) = if end_pos.line == pos.line {
                    (Ix::new(0), end_pos.column - pos.column)
                } else {
                    (end_pos.line - pos.line, end_pos.column)
                };
                Some(CursorChange {
                    pos,
                    kind: CursorChangeKind::Delete,
                    lines,
                    columns,
                })
            }
            Equal => None,
            Greater => Some(CursorChange {
                pos: self.pos_of_byte_pos(*byte_pos + *delete).unwrap(),
                kind: CursorChangeKind::Insert,
                lines: Ix::new(ins.chars().filter(|&c| c == '\n').count()),
                columns: if !ins.ends_with('\n')
                    && let Some(line) = ins.lines().next_back()
                {
                    line.graphemes().map(|g| g.columns()).sum()
                } else {
                    Ix::new(0)
                },
            }),
        }
    }
    pub fn indent_on_line(&self, line: Ix<Line>) -> Ix<Column> {
        let Some(line) = self.line(line) else {
            return Ix::new(0);
        };
        line.graphemes()
            .take_while(|g| g.is_whitespace())
            .map(|g| g.columns())
            .sum()
    }
    pub fn columns_in_line(&self, line: Ix<Line>) -> Ix<Column> {
        let Some(line) = self.line(line) else {
            return Ix::new(0);
        };
        line.graphemes().map(|g| g.columns()).sum()
    }
    pub fn line_has_content(&self, line: Ix<Line>) -> bool {
        let Some(line) = self.line(line) else {
            return false;
        };
        line.graphemes().any(|g| !g.is_whitespace())
    }

    pub fn graphemes_to_bytes(&self, graphemes: Ix<ix::Grapheme>) -> Option<Ix<Byte>> {
        for (g, (b, _)) in (Ix::new(0)..).zip(self.graphemes_with_bytes()) {
            if graphemes == g {
                return Some(b);
            }
        }
        None
    }

    pub fn column_range_to_byte_range(&self, column_range: Range<Ix<Column>>) -> Range<Ix<Byte>> {
        let mut start = None;
        let mut end = None;
        for (c, b) in self.columns_bytes() {
            for (x, i) in [
                (&mut start, column_range.start),
                (&mut end, column_range.end),
            ] {
                if x.is_none() && i == c {
                    *x = Some(b);
                }
            }
        }

        start.unwrap_or(self.byte_len())..end.unwrap_or(self.byte_len())
    }
}

impl<'a> tree_sitter::TextProvider<&'a str> for &'a Rope {
    type I = Chunks<'a>;

    fn text(&mut self, node: tree_sitter::Node) -> Self::I {
        let range = node.byte_range();
        self.byte_slice(Ix::new(range.start)..Ix::new(range.end))
            .unwrap()
            .chunks()
    }
}
