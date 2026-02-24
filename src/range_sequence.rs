use std::{
    fmt::{self, Debug},
    ops::{Add, Range, Sub},
};

use crate::aprintln::aprintln;

pub struct RelRange<R> {
    start_offset: R,
    len: R,
}

pub struct RangeSequence<R, T>(Vec<(RelRange<R>, T)>);
impl<R, T> Default for RangeSequence<R, T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<R, T> RangeSequence<R, T> {
    pub fn from_abs_ordered(ranges: impl IntoIterator<Item = (Range<R>, T)>) -> Self
    where
        R: Copy + Default + Sub<Output = R>,
    {
        let mut vec = Vec::new();
        let mut last = R::default();
        for (Range { start, end }, value) in ranges {
            let start_offset = start - last;
            let len = end - start;
            vec.push((RelRange { start_offset, len }, value));
            last = start;
        }
        Self(vec)
    }

    pub fn ranges(&self) -> impl Iterator<Item = (Range<R>, &T)>
    where
        R: Copy + Default + Add<Output = R>,
    {
        gen {
            let mut last = R::default();
            for (RelRange { start_offset, len }, value) in &self.0 {
                let start = *start_offset + last;
                let end = start + *len;
                last = start;
                yield (start..end, value);
            }
        }
    }

    /// This assumes that `R::default()` is 0.
    pub fn edit_insert(&mut self, pos: R, len: R)
    where
        R: Copy + Ord + Default + Add<Output = R> + Debug,
    {
        if len == R::default() {
            return;
        }
        let mut rel_start = R::default();
        for (
            RelRange {
                start_offset,
                len: range_len,
            },
            _,
        ) in &mut self.0
        {
            let start = *start_offset + rel_start;
            let end = start + *range_len;
            rel_start = start;
            if start > pos {
                *start_offset = *start_offset + len;
                break;
            }
            if end >= pos {
                *range_len = *range_len + len;
            }
        }
    }

    pub fn edit_delete(&mut self, pos: R, len: R)
    where
        R: Copy + Ord + Default + Add<Output = R> + Sub<Output = R>,
    {
        let mut rel_start = R::default();
        for (
            RelRange {
                start_offset,
                len: range_len,
            },
            _,
        ) in &mut self.0
        {
            let start = *start_offset + rel_start;
            let end = start + *range_len;
            rel_start = start;
            if start > pos {
                *start_offset = *start_offset - len.min(start - pos);
                break;
            }
            if end > pos {
                *range_len = *range_len - len.min(end - pos);
            }
        }
    }
}
