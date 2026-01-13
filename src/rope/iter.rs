use crate::{grapheme::Grapheme, rope::RopeSlice};

pub struct Lines<'a>(pub(super) crop::iter::Lines<'a>);

impl<'a> Iterator for Lines<'a> {
    type Item = RopeSlice<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(RopeSlice)
    }
}

impl<'a> DoubleEndedIterator for Lines<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(RopeSlice)
    }
}

pub struct Graphemes<'a>(pub(super) crop::iter::Graphemes<'a>);

impl<'a> Iterator for Graphemes<'a> {
    type Item = Grapheme;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|g| unsafe { Grapheme::new_unchecked(g) })
    }
}

impl<'a> DoubleEndedIterator for Graphemes<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0
            .next_back()
            .map(|g| unsafe { Grapheme::new_unchecked(g) })
    }
}
