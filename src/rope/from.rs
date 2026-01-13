use std::{borrow::Cow, str::FromStr};

use crop::iter::*;

use crate::rope::{Rope, RopeSlice};

impl<'a> From<&'a Rope> for Bytes<'a> {
    fn from(value: &'a Rope) -> Self {
        (&value.0).into()
    }
}
impl<'a> From<&'a Rope> for Chars<'a> {
    fn from(value: &'a Rope) -> Self {
        (&value.0).into()
    }
}
impl<'a> From<&'a Rope> for Chunks<'a> {
    fn from(value: &'a Rope) -> Self {
        (&value.0).into()
    }
}
impl<'a> From<&'a Rope> for Graphemes<'a> {
    fn from(value: &'a Rope) -> Self {
        (&value.0).into()
    }
}
impl<'a> From<&'a Rope> for Lines<'a> {
    fn from(value: &'a Rope) -> Self {
        (&value.0).into()
    }
}
impl<'a> From<&'a Rope> for RawLines<'a> {
    fn from(value: &'a Rope) -> Self {
        (&value.0).into()
    }
}

impl From<&str> for Rope {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}
impl From<Cow<'_, str>> for Rope {
    fn from(value: Cow<'_, str>) -> Self {
        Self(value.into())
    }
}
impl From<RopeSlice<'_>> for Rope {
    fn from(value: RopeSlice<'_>) -> Self {
        Self(value.0.into())
    }
}
impl From<String> for Rope {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}
impl FromStr for Rope {
    type Err = !;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Ok(s) = crop::Rope::from_str(s);
        Ok(Self(s))
    }
}

impl PartialEq<str> for Rope {
    fn eq(&self, other: &str) -> bool {
        self.0 == *other
    }
}
impl PartialEq<Rope> for str {
    fn eq(&self, other: &Rope) -> bool {
        *self == other.0
    }
}

impl PartialEq<&str> for Rope {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}
impl PartialEq<Rope> for &str {
    fn eq(&self, other: &Rope) -> bool {
        *self == other.0
    }
}

impl PartialEq<String> for Rope {
    fn eq(&self, other: &String) -> bool {
        self.0 == *other
    }
}
impl PartialEq<Rope> for String {
    fn eq(&self, other: &Rope) -> bool {
        *self == other.0
    }
}

impl PartialEq<Cow<'_, str>> for Rope {
    fn eq(&self, other: &Cow<'_, str>) -> bool {
        self.0 == *other
    }
}
impl PartialEq<Rope> for Cow<'_, str> {
    fn eq(&self, other: &Rope) -> bool {
        *self == other.0
    }
}

impl PartialEq<RopeSlice<'_>> for Rope {
    fn eq(&self, other: &RopeSlice<'_>) -> bool {
        self.0 == other.0
    }
}
impl PartialEq<Rope> for RopeSlice<'_> {
    fn eq(&self, other: &Rope) -> bool {
        self.0 == other.0
    }
}
