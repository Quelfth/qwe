use std::{cmp::Ordering::*, iter, mem, range::Range, str::FromStr};

use extension_traits::extension;
use tree_sitter::QueryPredicateArg;

use crate::{ix::{Byte, Ix, ix}, rope::Rope};


#[derive(Copy, Clone)]
pub enum Directive {
    Slice { range: Range<Offset> },
}

#[derive(Copy, Clone)]
pub struct Offset {
    from_end: bool,
    offset: i32,
}

impl FromStr for Offset {
    type Err = DirectiveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let (se, value) = s.split_at(s.ceil_char_boundary(1));
        let from_end = match se {
            "s" => false,
            "e" => true,
            _ => Err(DirectiveError)?,
        };
        Ok(Self {
            from_end,
            offset: value.trim().parse().map_err(|_| DirectiveError)?,
        })
    }
}

impl Offset {
    pub fn apply(self, range: Range<Ix<Byte>>, text: &Rope) -> Ix<Byte> {
        let mut value = if self.from_end { range.end } else { range.start };
        match self.offset.cmp(&0) {
            Less => for char in text.byte_slice(..value).unwrap().chars().rev().take((-self.offset) as _) {
                value -= ix(char.len_utf8());
            },
            Equal => (),
            Greater => for char in text.byte_slice(value..).unwrap().chars().take(self.offset as _) {
                value += ix(char.len_utf8());
            },
        }
        value
    }
}

#[extension(trait RangeOffsetExt)]
impl Range<Offset> {
    fn apply(self, range: Range<Ix<Byte>>, text: &Rope) -> Range<Ix<Byte>> {
        Range {
            start: self.start.apply(range, text),
            end: self.end.apply(range, text),
        }
    }
}

pub struct DirectiveError;

impl Directive {
    pub fn parse(name: &str, args: &[QueryPredicateArg]) -> Result<(u32, Self), DirectiveError> {
        let mut args = args.iter();

        let Some(&QueryPredicateArg::Capture(capture)) = args.next() else {
            return Err(DirectiveError);
        };
        Ok((capture, match name {
            "slice" => {

                let Some(QueryPredicateArg::String(start)) = args.next() else {
                    return Err(DirectiveError);
                };

                let start = start.parse()?;

                let Some(QueryPredicateArg::String(end)) = args.next() else {
                    return Err(DirectiveError);
                };

                let end = end.parse()?;
                
                Directive::Slice {
                    range: start..end,
                }
            }
            _ => Err(DirectiveError)?,
        }))
    }

    pub fn apply_to_range(&self, range: Range<Ix<Byte>>, text: &Rope) -> impl Iterator<Item = Range<Ix<Byte>>> {
        match self {
            Self::Slice { range: slice } => iter::once(slice.apply(range, text)),
        }
    }
}

#[extension(pub trait DirectivesExt)]
impl &[Directive] {
    fn apply_to_range(self, range: Range<Ix<Byte>>, text: &Rope) -> Vec<Range<Ix<Byte>>> {
        let mut a = vec![range];
        let mut b = Vec::new();

        for directive in self {
            b.extend(a.iter().flat_map(|&r| directive.apply_to_range(r, text)));
            mem::swap(&mut a, &mut b);
            b.clear();
        }

        a
    }
}
