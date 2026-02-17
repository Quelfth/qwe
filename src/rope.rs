use std::ops::RangeBounds;

use crate::ix::Ix;

mod display;
mod from;
mod iter;
mod rope;
mod slice;
#[cfg(test)]
mod test;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Rope(crop::Rope);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RopeSlice<'a>(crop::RopeSlice<'a>);

impl<'a> From<crop::RopeSlice<'a>> for RopeSlice<'a> {
    fn from(value: crop::RopeSlice<'a>) -> Self {
        Self(value)
    }
}

#[inline]
fn range_bounds_to_start_end<U>(
    range: impl RangeBounds<Ix<U, usize>>,
    lo: Ix<U, usize>,
    hi: Ix<U, usize>,
) -> (Ix<U, usize>, Ix<U, usize>) {
    use core::ops::Bound;

    let start = match range.start_bound() {
        Bound::Included(&n) => n,
        Bound::Excluded(&n) => n + Ix::new(1),
        Bound::Unbounded => lo,
    };

    let end = match range.end_bound() {
        Bound::Included(&n) => n + Ix::new(1),
        Bound::Excluded(&n) => n,
        Bound::Unbounded => hi,
    };

    (start, end)
}
