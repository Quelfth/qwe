use std::{marker::PhantomData, ops::RangeBounds};

mod ops;

pub struct Ix<U, T = usize>(T, PhantomData<U>);

impl<U, T: std::fmt::Debug> std::fmt::Debug for Ix<U, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Ix").field(&self.0).finish()
    }
}

impl<U, T: Default> Default for Ix<U, T> {
    fn default() -> Self {
        Self(T::default(), PhantomData)
    }
}

impl<U, T: Copy> Copy for Ix<U, T> {}
impl<U, T: Clone> Clone for Ix<U, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<U, T> Ix<U, T> {
    pub const fn new(value: T) -> Self {
        Self(value, PhantomData)
    }

    pub const fn inner_ref(&self) -> &T {
        &self.0
    }

    pub fn inner(self) -> T {
        self.0
    }
}

impl<U> Ix<U> {
    pub fn saturating_sub(self, other: Self) -> Self {
        Self::new(self.0.saturating_sub(other.0))
    }
    pub fn checked_sub(self, other: Self) -> Option<Self> {
        Some(Self::new(self.0.checked_sub(other.0)?))
    }

    pub const ZERO: Self = Ix(0, PhantomData);
}

impl<U> Ix<U, isize> {
    pub fn to_usize(self) -> Ix<U> {
        Ix::new(self.0 as _)
    }
}

pub enum Line {}
pub enum Column {}
pub enum Byte {}
pub enum Utf16 {}
pub enum Grapheme {}

macro_rules! ixto {
    ($ix:ident) => {
        let $ix = $ix.inner();
    };
}
pub(crate) use ixto;

pub struct MappedRange<U, R>(R, PhantomData<U>);

impl<U, R> MappedRange<U, R> {
    pub fn new(value: R) -> Self {
        Self(value, PhantomData)
    }
}

impl<U, T, R: RangeBounds<Ix<U, T>>> RangeBounds<T> for MappedRange<U, R> {
    fn start_bound(&self) -> std::ops::Bound<&T> {
        self.0.start_bound().map(|i| i.inner_ref())
    }

    fn end_bound(&self) -> std::ops::Bound<&T> {
        self.0.end_bound().map(|i| i.inner_ref())
    }
}
