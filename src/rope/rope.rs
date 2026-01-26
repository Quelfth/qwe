use std::ops::RangeBounds;

use crop::iter::{Bytes, Chars, Chunks, RawLines};

use crate::{
    aprintln::{aprint, aprintln},
    document::{Change, CursorChange, CursorChangeKind},
    pos::Pos,
    rope::iter::{Graphemes, Lines},
};

use super::{Rope, RopeSlice, range_bounds_to_start_end};

use std::cmp::Ordering::*;

impl Rope {
    #[must_use]
    fn validate_byte_range(&self, byte_range: &(impl RangeBounds<usize> + Clone)) -> Option<()> {
        let (start, end) = range_bounds_to_start_end(byte_range.clone(), 0, self.byte_len());
        if start > end {
            return None;
        }
        if end > self.byte_len() {
            return None;
        }
        if !self.0.is_char_boundary(start) || !self.0.is_char_boundary(end) {
            return None;
        }

        Some(())
    }

    #[must_use]
    fn validate_byte_offset(&self, byte_offset: usize) -> Option<()> {
        (byte_offset <= self.0.byte_len() && self.0.is_char_boundary(byte_offset)).then_some(())
    }

    fn validate_line_range(&self, line_range: &(impl RangeBounds<usize> + Clone)) -> Option<()> {
        let (start, end) = range_bounds_to_start_end(line_range.clone(), 0, self.0.line_len());
        if start > end {
            return None;
        }
        if end > self.0.line_len() {
            return None;
        }

        Some(())
    }

    pub fn byte(&self, i: usize) -> Option<u8> {
        (i < self.0.byte_len()).then(|| self.0.byte(i))
    }

    pub fn byte_len(&self) -> usize {
        self.0.byte_len()
    }

    pub fn byte_of_line(&self, line_offset: usize) -> Option<usize> {
        (line_offset <= self.0.line_len()).then(|| self.0.byte_of_line(line_offset))
    }

    pub fn byte_slice(&self, byte_range: impl RangeBounds<usize> + Clone) -> Option<RopeSlice<'_>> {
        self.validate_byte_range(&byte_range)?;
        Some(self.0.byte_slice(byte_range).into())
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
    pub fn delete(&mut self, byte_range: impl RangeBounds<usize> + Clone) -> Option<()> {
        self.validate_byte_range(&byte_range)?;
        self.0.delete(byte_range);
        Some(())
    }

    pub fn graphemes(&self) -> Graphemes<'_> {
        Graphemes(self.0.graphemes())
    }

    #[must_use]
    pub fn insert(&mut self, byte_offset: usize, text: impl AsRef<str>) -> Option<()> {
        self.validate_byte_offset(byte_offset)?;
        self.0.insert(byte_offset, text);
        Some(())
    }

    pub fn is_char_boundary(&self, byte_offset: usize) -> Option<bool> {
        if byte_offset > self.byte_len() {
            return None;
        }
        Some(self.0.is_char_boundary(byte_offset))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn is_grapheme_boundary(&self, byte_offset: usize) -> Option<bool> {
        if byte_offset > self.byte_len() {
            return None;
        }
        Some(self.0.is_grapheme_boundary(byte_offset))
    }

    pub fn line(&self, line_index: usize) -> Option<RopeSlice<'_>> {
        if line_index >= self.0.line_len() {
            return None;
        }
        Some(self.0.line(line_index).into())
    }

    pub fn line_len(&self) -> usize {
        self.0.line_len()
    }

    pub fn line_count(&self) -> usize {
        let len = self.0.line_len();
        match self.0.chars().next_back() {
            Some('\n') => len,
            None => 0,
            _ => len - 1,
        }
    }

    pub fn line_of_byte(&self, byte_offset: usize) -> Option<usize> {
        if byte_offset > self.0.byte_len() {
            return None;
        }
        Some(self.0.line_of_byte(byte_offset))
    }

    pub fn line_slice(&self, line_range: impl RangeBounds<usize> + Clone) -> Option<RopeSlice<'_>> {
        self.validate_line_range(&line_range)?;
        Some(self.0.line_slice(line_range).into())
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
        byte_range: impl RangeBounds<usize> + Clone,
        text: impl AsRef<str>,
    ) -> Option<()> {
        self.validate_byte_range(&byte_range)?;
        self.0.replace(byte_range, text);
        Some(())
    }

    pub fn chunk_from_byte(&self, byte: usize) -> Option<&str> {
        Some(self.byte_slice(byte..)?.chunks().next().unwrap_or(""))
    }

    pub fn ts_callback<'a>(&'a self) -> impl Fn(usize, tree_sitter::Point) -> &'a str {
        |byte, _| {
            self.chunk_from_byte(byte)
                .expect("tree sitter should be providing valid byte offsets")
        }
    }

    pub fn ts_pos_of_byte(&self, byte: usize) -> Option<tree_sitter::Point> {
        let line = self.line_of_byte(byte)?;
        Some(tree_sitter::Point {
            row: line,
            column: byte - line,
        })
    }

    pub fn pos_of_byte_pos(&self, byte_pos: usize) -> Option<Pos> {
        let line = self.line_of_byte(byte_pos)?;
        let line_byte = self.byte_of_line(line)?;
        let byte_in_line = byte_pos - line_byte;
        let column = self
            .line(line)?
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
        let insert = insert.len();
        match insert.cmp(delete) {
            Less => {
                let byte_pos = byte_pos + insert;
                let delete = delete - insert;
                let pos = self.pos_of_byte_pos(byte_pos).unwrap();
                let end_pos = self.pos_of_byte_pos(byte_pos + delete).unwrap();
                let (lines, bytes) = if end_pos.line == pos.line {
                    (0, delete)
                } else {
                    (
                        end_pos.line - pos.line,
                        byte_pos + delete - self.byte_of_line(end_pos.line).unwrap(),
                    )
                };
                Some(CursorChange {
                    pos,
                    kind: CursorChangeKind::Delete,
                    lines,
                    columns: bytes,
                })
            }
            Equal => None,
            Greater => Some(CursorChange {
                pos: self.pos_of_byte_pos(byte_pos + delete).unwrap(),
                kind: CursorChangeKind::Insert,
                lines: ins.chars().filter(|&c| c == '\n').count(),
                columns: if !ins.ends_with('\n')
                    && let Some(line) = ins.lines().next_back()
                {
                    line.len()
                } else {
                    0
                },
            }),
        }
    }
}

impl<'a> tree_sitter::TextProvider<&'a str> for &'a Rope {
    type I = Chunks<'a>;

    fn text(&mut self, node: tree_sitter::Node) -> Self::I {
        self.byte_slice(node.byte_range()).unwrap().chunks()
    }
}
