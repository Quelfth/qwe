use std::ops::{Range, RangeBounds};

use crop::iter::{Bytes, Chars, Chunks, RawLines};

use crate::{
    grapheme::Grapheme,
    rope::{
        RopeSlice,
        iter::{Graphemes, Lines},
        range_bounds_to_start_end,
    },
};

impl<'a> RopeSlice<'a> {
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
        if end > self.line_len() {
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

    pub fn byte_slice(&self, byte_range: impl RangeBounds<usize> + Clone) -> Option<RopeSlice<'a>> {
        self.validate_byte_range(&byte_range)?;
        Some(self.0.byte_slice(byte_range).into())
    }

    pub fn bytes(&self) -> Bytes<'a> {
        self.0.bytes()
    }

    pub fn chars(&self) -> Chars<'a> {
        self.0.chars()
    }

    pub fn chunks(&self) -> Chunks<'a> {
        self.0.chunks()
    }

    pub fn graphemes(&self) -> Graphemes<'a> {
        Graphemes(self.0.graphemes())
    }

    pub fn graphemes_with_bytes(&self) -> impl Iterator<Item = (usize, Grapheme)> {
        let iter = self.graphemes();
        gen {
            let mut byte = 0;

            for grapheme in iter {
                let len = grapheme.len();
                yield (byte, grapheme);
                byte += len;
            }
        }
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

    pub fn line(&self, line_index: usize) -> Option<RopeSlice<'a>> {
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
            Some('\n') => len - 1,
            None => 0,
            _ => len,
        }
    }

    pub fn line_of_byte(&self, byte_offset: usize) -> Option<usize> {
        if byte_offset > self.0.byte_len() {
            return None;
        }
        Some(self.0.line_of_byte(byte_offset))
    }

    pub fn line_slice(&self, line_range: impl RangeBounds<usize> + Clone) -> Option<RopeSlice<'a>> {
        self.validate_line_range(&line_range)?;
        Some(self.0.line_slice(line_range).into())
    }

    pub fn lines(&self) -> Lines<'a> {
        Lines(self.0.lines())
    }

    pub fn raw_lines(&self) -> RawLines<'a> {
        self.0.raw_lines()
    }

    pub fn columns_bytes(&self) -> impl Iterator<Item = (usize, usize)> {
        let iter = self.graphemes();
        gen {
            let mut column = 0;
            let mut byte = 0;

            for grapheme in iter {
                let columns = grapheme.columns();
                let bytes = grapheme.len();
                for _ in 0..columns {
                    yield (column, byte);
                    column += 1;
                }
                byte += bytes;
            }
        }
    }

    pub fn column_range_to_byte_range(&self, column_range: Range<usize>) -> Range<usize> {
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
