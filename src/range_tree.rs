use std::{
    collections::BTreeMap,
    iter::{self, Sum},
    ops::{Add, Range, Sub},
};

use auto_enums::auto_enum;

use crate::ix::Ix;

pub struct RangeTree<R, T>(RelRangeTree<R, T>);

impl<R, T> RangeTree<R, T>
where
    R: Default + Copy + Sub<Output = R> + Add<Output = R> + DivideUsize + Sum + Ord,
{
    pub fn build(values: Vec<(Range<R>, T)>) -> Self {
        Self(RelRangeTree::build(
            values,
            R::default(),
            RelativeDirection::Right,
        ))
    }
}

impl<R, T> Default for RangeTree<R, T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<R, T> RangeTree<R, T>
where
    R: Copy + Default + Sub<Output = R> + Add<Output = R> + Ord,
{
    pub fn overlapping(&self, range: Range<R>) -> impl Iterator<Item = &T> {
        RangeTreeRef {
            parent_center: R::default(),
            direction: RelativeDirection::Right,
            rel_ref: &self.0,
        }
        .overlapping(range)
    }
}

#[derive(Copy, Clone)]
enum RelativeDirection {
    Left,
    Right,
}

impl RelativeDirection {
    fn rel_to_abs<R: Copy + Add<Output = R> + Sub<Output = R>>(&self, center: R, rel: R) -> R {
        match self {
            RelativeDirection::Left => center - rel,
            RelativeDirection::Right => center + rel,
        }
    }

    fn abs_to_rel<R: Copy + Sub<Output = R>>(&self, center: R, abs: R) -> R {
        match self {
            RelativeDirection::Left => center - abs,
            RelativeDirection::Right => abs - center,
        }
    }
}

macro_rules! def_abs_ref {
    ($name:ident, $inner:ident) => {
        struct $name<'a, R: Copy, T> {
            parent_center: R,
            direction: RelativeDirection,
            rel_ref: &'a $inner<R, T>,
        }

        impl<'a, R: Copy, T> $name<'a, R, T> {
            pub fn rel(self) -> &'a $inner<R, T> {
                self.rel_ref
            }
        }
        impl<'a, R: Copy, T> Copy for $name<'a, R, T> {}
        impl<'a, R: Copy, T> Clone for $name<'a, R, T> {
            fn clone(&self) -> Self {
                *self
            }
        }
    };
}

def_abs_ref!(RangeTreeRef, RelRangeTree);
def_abs_ref!(RangeTreeInnerRef, RelRangeTreeInner);
def_abs_ref!(RangeTreeCenterRef, RelRangeTreeCenter);

pub struct RelRangeTree<R, T>(Option<Box<RelRangeTreeInner<R, T>>>);
impl<R, T> Default for RelRangeTree<R, T> {
    fn default() -> Self {
        Self(None)
    }
}

struct RelRangeTreeInner<R, T> {
    center: RelRangeTreeCenter<R, T>,
    left: RelRangeTree<R, T>,
    right: RelRangeTree<R, T>,
}

struct RelRangeTreeCenter<R, T> {
    center: R,
    data: Vec<T>,
    starts: BTreeMap<R, usize>,
    ends: BTreeMap<R, usize>,
}

pub trait DivideUsize {
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

impl<R, T> RelRangeTree<R, T> {
    fn build(values: Vec<(Range<R>, T)>, parent_center: R, direction: RelativeDirection) -> Self
    where
        R: Copy + Ord + Sum + DivideUsize + Sub<Output = R>,
    {
        if values.is_empty() {
            return Self(None);
        }
        let abs_center = values
            .iter()
            .flat_map(|r| [r.0.start, r.0.end])
            .sum::<R>()
            .divide_usize(values.len() * 2);

        let mut left = Vec::new();
        let mut right = Vec::new();

        let mut data = Vec::new();
        let mut starts = BTreeMap::new();
        let mut ends = BTreeMap::new();

        for range in values {
            if range.0.end < abs_center {
                left.push(range);
            } else if range.0.start > abs_center {
                right.push(range);
            } else {
                let (Range { start, end }, datum) = range;
                let i = data.len();
                data.push(datum);
                starts.insert(RelativeDirection::Left.abs_to_rel(abs_center, start), i);
                ends.insert(RelativeDirection::Right.abs_to_rel(abs_center, end), i);
            }
        }

        Self(Some(Box::new(RelRangeTreeInner {
            center: RelRangeTreeCenter {
                center: direction.abs_to_rel(parent_center, abs_center),
                data,
                starts,
                ends,
            },
            left: Self::build(
                left.into_iter().collect(),
                abs_center,
                RelativeDirection::Left,
            ),
            right: Self::build(
                right.into_iter().collect(),
                abs_center,
                RelativeDirection::Right,
            ),
        })))
    }
}

impl<'a, R: Copy, T> RangeTreeRef<'a, R, T> {
    pub fn inner(&self) -> Option<RangeTreeInnerRef<'a, R, T>> {
        self.rel().0.as_deref().map(|inner| RangeTreeInnerRef {
            parent_center: self.parent_center,
            direction: self.direction,
            rel_ref: inner,
        })
    }

    pub fn overlapping(self, range: Range<R>) -> Box<dyn Iterator<Item = &'a T> + 'a>
    where
        R: Ord + Add<Output = R> + Sub<Output = R>,
    {
        match &self.inner() {
            Some(inner) => Box::new(inner.overlapping(range)),
            None => Box::new(iter::empty()),
        }
    }
}

impl<'a, R: Copy, T> RangeTreeInnerRef<'a, R, T> {
    fn center(&self) -> RangeTreeCenterRef<'a, R, T> {
        RangeTreeCenterRef {
            parent_center: self.parent_center,
            direction: self.direction,
            rel_ref: &self.rel().center,
        }
    }

    fn left(&self) -> RangeTreeRef<'a, R, T>
    where
        R: Copy + Sub<Output = R> + Add<Output = R>,
    {
        RangeTreeRef {
            parent_center: self.center().center_point(),
            direction: RelativeDirection::Left,
            rel_ref: &self.rel().left,
        }
    }

    fn right(&self) -> RangeTreeRef<'a, R, T>
    where
        R: Copy + Sub<Output = R> + Add<Output = R>,
    {
        RangeTreeRef {
            parent_center: self.center().center_point(),
            direction: RelativeDirection::Right,
            rel_ref: &self.rel().right,
        }
    }
}

impl<'a, R: Copy, T> RangeTreeInnerRef<'a, R, T> {
    #[auto_enum(Iterator)]
    pub fn overlapping(self, range: Range<R>) -> impl Iterator<Item = &'a T>
    where
        R: Ord + Copy + Sub<Output = R> + Add<Output = R>,
    {
        if range.end < self.center().center_point() {
            self.left()
                .overlapping(range.clone())
                .chain(self.center().starting_before(range.end))
        } else if range.start > self.center().center_point() {
            self.right()
                .overlapping(range.clone())
                .chain(self.center().ending_after(range.start))
        } else {
            self.left()
                .overlapping(range.clone())
                .chain(self.center().rel().data.iter())
                .chain(self.right().overlapping(range.clone()))
        }
    }
}

impl<'a, R: Copy, T> RangeTreeCenterRef<'a, R, T> {
    pub fn center_point(self) -> R
    where
        R: Sub<Output = R> + Add<Output = R>,
    {
        self.direction
            .rel_to_abs(self.parent_center, self.rel().center)
    }

    pub fn starting_before(self, point: R) -> impl Iterator<Item = &'a T>
    where
        R: Ord + Add<Output = R> + Sub<Output = R>,
    {
        self.rel()
            .starts
            .range((self.center_point() - point)..)
            .map(move |(_, &i)| &self.rel().data[i])
    }

    pub fn ending_after(self, point: R) -> impl Iterator<Item = &'a T>
    where
        R: Ord + Add<Output = R> + Sub<Output = R>,
    {
        self.rel()
            .ends
            .range((point - self.center_point())..)
            .map(move |(_, &i)| &self.rel().data[i])
    }
}
