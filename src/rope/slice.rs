use std::ops::{Bound, RangeBounds};

use crop::iter::{Bytes, Chars, Chunks, RawLines};

use crate::{
    grapheme::Grapheme,
    ix::{Byte, Column, Ix, Line, MappedRange, Utf16, ixto},
    rope::{
        RopeSlice,
        iter::{Graphemes, Lines},
        range_bounds_to_start_end,
    },
    util::MapBounds,
};

impl<'a> RopeSlice<'a> {
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
            range_bounds_to_start_end(line_range.clone(), Ix::new(0), self.line_len());
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

    pub fn byte_len(&self) -> Ix<Byte> {
        Ix::new(self.0.byte_len())
    }

    pub fn byte_of_line(&self, line_offset: usize) -> Option<usize> {
        (line_offset <= self.0.line_len()).then(|| self.0.byte_of_line(line_offset))
    }

    pub fn byte_slice(
        &self,
        byte_range: impl RangeBounds<Ix<Byte>> + Clone,
    ) -> Option<RopeSlice<'a>> {
        self.validate_byte_range(&byte_range)?;
        let start = match byte_range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => *start + Ix::new(1),
            Bound::Unbounded => Ix::new(0),
        }
        .inner();
        let end = match byte_range.end_bound() {
            Bound::Included(end) => *end + Ix::new(1),
            Bound::Excluded(end) => *end,
            Bound::Unbounded => self.byte_len(),
        }
        .inner();
        Some(self.0.byte_slice(start..end).into())
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

    pub fn line(&self, line_index: usize) -> Option<RopeSlice<'a>> {
        if line_index >= self.0.line_len() {
            return None;
        }
        Some(self.0.line(line_index).into())
    }

    pub fn line_len(&self) -> Ix<Line> {
        Ix::new(self.0.line_len())
    }

    pub fn line_count(&self) -> usize {
        let len = self.0.line_len();
        match self.0.chars().next_back() {
            Some('\n') => len - 1,
            None => 0,
            _ => len,
        }
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
    ) -> Option<RopeSlice<'a>> {
        self.validate_line_range(&line_range)?;
        Some(self.0.line_slice(MappedRange::new(line_range)).into())
    }

    pub fn lines(&self) -> Lines<'a> {
        Lines(self.0.lines())
    }

    pub fn raw_lines(&self) -> RawLines<'a> {
        self.0.raw_lines()
    }

    pub fn utf16_of_byte(&self, byte: Ix<Byte>) -> Option<Ix<Utf16>> {
        self.validate_byte_offset(byte)?;
        Some(Ix::new(self.0.utf16_code_unit_of_byte(byte.inner())))
    }

    pub fn byte_of_utf16(&self, utf16: Ix<Utf16>) -> Option<Ix<Byte>> {
        if utf16.inner() > self.0.utf16_len() {
            return None;
        }
        Some(Ix::new(self.0.byte_of_utf16_code_unit(utf16.inner())))
    }

    pub fn byte_of_utf16_saturating(&self, utf16: Ix<Utf16>) -> Ix<Byte> {
        self.byte_of_utf16(utf16).unwrap_or(self.byte_len())
    }

    pub fn columns_bytes(&self) -> impl Iterator<Item = (Ix<Column>, Ix<Byte>)> {
        let iter = self.graphemes();
        gen {
            let mut column = Ix::new(0);
            let mut byte = Ix::new(0);

            for grapheme in iter {
                let columns = grapheme.columns();
                let bytes = grapheme.len();
                for _ in Ix::new(0)..columns {
                    yield (column, byte);
                    column += Ix::new(1);
                }
                byte += bytes;
            }
        }
    }

    pub fn column_range_to_byte_range<R: MapBounds<Ix<Column>, Ix<Byte>>>(
        &self,
        column_range: R,
    ) -> R::Out {
        column_range.map_bounds(|b| self.columns_to_bytes(b))
        // let mut start = None;
        // let mut end = None;
        // for (c, b) in self.columns_bytes() {
        //     for (x, i) in [
        //         (&mut start, column_range.start),
        //         (&mut end, column_range.end),
        //     ] {
        //         if x.is_none() && i == c {
        //             *x = Some(b);
        //         }
        //     }
        // }

        // start.unwrap_or(self.byte_len())..end.unwrap_or(self.byte_len())
    }

    pub fn columns_to_bytes(&self, columns: Ix<Column>) -> Ix<Byte> {
        match self.columns_to_bytes_strict(columns) {
            Ok(x) | Err(x) => x,
        }
    }

    pub fn columns_to_bytes_strict(&self, columns: Ix<Column>) -> Result<Ix<Byte>, Ix<Byte>> {
        let mut byte = None;
        let mut c_len = Ix::new(0);
        for (c, b) in self.columns_bytes() {
            let (x, i) = (&mut byte, columns);
            if x.is_none() && i == c {
                *x = Some(b);
            }
            c_len = c + Ix::new(1);
        }
        match byte {
            Some(b) => Ok(b),
            None => {
                if columns == c_len {
                    Ok(self.byte_len())
                } else {
                    Err(self.byte_len())
                }
            }
        }
    }

    pub fn column_count(&self) -> Ix<Column> {
        self.graphemes().map(|g| g.columns()).sum()
    }

    pub fn column_slice(
        &self,
        column_range: impl MapBounds<Ix<Column>, Ix<Byte>, Out: RangeBounds<Ix<Byte>> + Clone>,
    ) -> RopeSlice<'a> {
        let Some(slice) = self.byte_slice(self.column_range_to_byte_range(column_range)) else {
            panic!("column range end was greater than the start",);
        };
        slice
    }
}
