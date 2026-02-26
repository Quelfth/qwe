use super::Ix;
use std::{
    hash::{Hash, Hasher},
    iter::{Step, Sum},
    ops::{Add, AddAssign, Div, Mul, Neg, Rem, Sub, SubAssign},
};

macro_rules! op {
    ($lhs:ident $op:tt $rhs:ident) => {
        Ix($lhs.0 $op $rhs.0, std::marker::PhantomData)
    };
}

impl<U, T: Add<T>> Add for Ix<U, T> {
    type Output = Ix<U, <T as Add>::Output>;

    fn add(self, rhs: Self) -> Self::Output {
        op!(self + rhs)
    }
}

impl<U, T: Sub<T>> Sub for Ix<U, T> {
    type Output = Ix<U, <T as Sub>::Output>;

    fn sub(self, rhs: Self) -> Self::Output {
        op!(self - rhs)
    }
}

impl<U, T: AddAssign> AddAssign for Ix<U, T> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}
impl<U, T: SubAssign> SubAssign for Ix<U, T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}

impl<U, T: Neg> Neg for Ix<U, T> {
    type Output = Ix<U, <T as Neg>::Output>;

    fn neg(self) -> Self::Output {
        Ix::new(-self.0)
    }
}

impl<U, T: Div> Div<T> for Ix<U, T> {
    type Output = Ix<U, <T as Div>::Output>;

    fn div(self, rhs: T) -> Self::Output {
        Ix::new(self.0 / rhs)
    }
}
impl<U, T: Mul> Mul<T> for Ix<U, T> {
    type Output = Ix<U, <T as Mul>::Output>;

    fn mul(self, rhs: T) -> Self::Output {
        Ix::new(self.0 * rhs)
    }
}
impl<U, T: Rem> Rem<T> for Ix<U, T> {
    type Output = Ix<U, <T as Rem>::Output>;

    fn rem(self, rhs: T) -> Self::Output {
        Ix::new(self.0 % rhs)
    }
}

impl<U, T: PartialEq> PartialEq for Ix<U, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<U, T: Eq> Eq for Ix<U, T> {}

impl<U, T: Hash> Hash for Ix<U, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<U, T: PartialOrd> PartialOrd for Ix<U, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<U, T: Ord> Ord for Ix<U, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<U, T: Sum> Sum for Ix<U, T> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Ix::new(iter.map(|i| i.inner()).sum())
    }
}

impl<U, T: Step> Step for Ix<U, T> {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        T::steps_between(start.inner_ref(), end.inner_ref())
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        T::forward_checked(start.inner(), count).map(Ix::new)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        T::backward_checked(start.inner(), count).map(Ix::new)
    }
}
