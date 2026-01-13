mod display;
mod from;
mod iter;
mod rope;
mod slice;

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
fn range_bounds_to_start_end<T, B>(range: B, lo: usize, hi: usize) -> (usize, usize)
where
    B: core::ops::RangeBounds<T>,
    T: core::ops::Add<usize, Output = usize> + Into<usize> + Copy,
{
    use core::ops::Bound;

    let start = match range.start_bound() {
        Bound::Included(&n) => n.into(),
        Bound::Excluded(&n) => n + 1,
        Bound::Unbounded => lo,
    };

    let end = match range.end_bound() {
        Bound::Included(&n) => n + 1,
        Bound::Excluded(&n) => n.into(),
        Bound::Unbounded => hi,
    };

    (start, end)
}
