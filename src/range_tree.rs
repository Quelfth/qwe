use std::{
    cmp::Ordering,
    collections::BTreeMap,
    iter::{self, Sum},
    ops::Range,
};

use auto_enums::auto_enum;

use crate::ix::Ix;

pub struct RangeTree<R, T>(Option<Box<RangeTreeInner<R, T>>>);
impl<R, T> Default for RangeTree<R, T> {
    fn default() -> Self {
        Self(None)
    }
}

struct RangeTreeInner<R, T> {
    center: RangeTreeCenter<R, T>,
    left: RangeTree<R, T>,
    right: RangeTree<R, T>,
}

struct RangeTreeCenter<R, T> {
    center: R,
    data: Vec<T>,
    starts: BTreeMap<R, usize>,
    ends: BTreeMap<R, usize>,
}

trait DivideUsize {
    fn divide_usize(self, usize: usize) -> Self;
}

impl DivideUsize for usize {
    fn divide_usize(self, usize: usize) -> Self {
        self / usize
    }
}

impl<U> DivideUsize for Ix<U> {
    fn divide_usize(self, usize: usize) -> Self {
        Ix::new(self.inner() / usize)
    }
}

impl<R, T> FromIterator<(Range<R>, T)> for RangeTree<R, T>
where
    R: Sum + DivideUsize + Ord + Copy,
{
    fn from_iter<I: IntoIterator<Item = (Range<R>, T)>>(iter: I) -> Self {
        let ranges = iter.into_iter().collect::<Vec<_>>();
        if ranges.is_empty() {
            return Self(None);
        }
        let c = ranges
            .iter()
            .flat_map(|r| [r.0.start, r.0.end])
            .sum::<R>()
            .divide_usize(ranges.len() * 2);

        let mut left = Vec::new();
        let mut right = Vec::new();

        let mut data = Vec::new();
        let mut starts = BTreeMap::new();
        let mut ends = BTreeMap::new();

        for range in ranges {
            if range.0.end < c {
                left.push(range);
            } else if range.0.start > c {
                right.push(range);
            } else {
                let (Range { start, end }, datum) = range;
                let i = data.len();
                data.push(datum);
                starts.insert(start, i);
                ends.insert(end, i);
            }
        }

        Self(Some(Box::new(RangeTreeInner {
            center: RangeTreeCenter {
                center: c,
                data,
                starts,
                ends,
            },
            left: left.into_iter().collect(),
            right: right.into_iter().collect(),
        })))
    }
}

impl<R: Ord + Copy, T> RangeTree<R, T> {
    pub fn overlapping(&self, range: Range<R>) -> Box<dyn Iterator<Item = &T> + '_> {
        match &self.0 {
            Some(inner) => Box::new(inner.overlapping(range)),
            None => Box::new(iter::empty()),
        }
    }
}

impl<R: Ord + Copy, T> RangeTreeInner<R, T> {
    #[auto_enum(Iterator)]
    pub fn overlapping(&self, range: Range<R>) -> impl Iterator<Item = &T> {
        if range.end < self.center.center {
            self.left
                .overlapping(range.clone())
                .chain(self.center.starting_before(range.end))
        } else if range.start > self.center.center {
            self.right
                .overlapping(range.clone())
                .chain(self.center.ending_after(range.start))
        } else {
            self.left
                .overlapping(range.clone())
                .chain(self.center.data.iter())
                .chain(self.right.overlapping(range.clone()))
        }
    }
}

impl<R: Ord, T> RangeTreeCenter<R, T> {
    pub fn starting_before(&self, point: R) -> impl Iterator<Item = &T> {
        self.starts.range(..=point).map(|(_, &i)| &self.data[i])
    }

    pub fn ending_after(&self, point: R) -> impl Iterator<Item = &T> {
        self.ends.range(point..).map(|(_, &i)| &self.data[i])
    }
}
