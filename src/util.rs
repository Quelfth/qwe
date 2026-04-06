use std::{
    borrow::Borrow,
    fmt::Write,
    ops::{Range, RangeFrom, RangeFull, RangeTo},
};

use extension_trait::extension_trait;
use tree_sitter::Node;

use crate::{
    grapheme::GraphemeExt,
    ix::{Column, Ix, Line},
};

pub fn leak<T>(value: T) -> &'static T {
    Box::leak(Box::new(value))
}

pub fn flip<T, U>((t, u): (T, U)) -> (U, T) { (u, t) }

pub fn pretty_node(node: Node<'_>) -> String {
    let mut string = String::new();
    format_node(node, 0, None, &mut string);
    string
}

fn format_node(node: Node<'_>, indent: usize, field_name: Option<&str>, out: &mut String) {
    let indent_str = "    ".repeat(indent);
    out.push_str(&indent_str);
    if let Some(field_name) = field_name {
        out.write_fmt(format_args!("{field_name}: ")).unwrap();
    }
    if !node.is_named() {
        let mut text = String::new();
        for c in node.kind().chars() {
            match c {
                '\\' => text.push_str("\\\\"),
                '"' => text.push_str("\\\""),
                c => text.push(c),
            }
        }
        out.write_fmt(format_args!("\"{text}\"")).unwrap();
        return;
    }
    out.write_fmt(format_args!("({}", node.kind())).unwrap();

    if node.child_count() == 0 {
        out.push(')');
        return;
    }

    out.push('\n');

    let mut cursor = node.walk();
    for (i, child) in (0..).zip(node.children(&mut cursor)) {
        let field_name = node.field_name_for_child(i);
        format_node(child, indent + 1, field_name, out);
        out.push('\n');
    }

    out.push_str(&format!("{})", indent_str));
}

pub trait MapBounds<T, U> {
    type Out;

    fn map_bounds(self, map: impl Fn(T) -> U) -> Self::Out;
}

impl<T, U> MapBounds<T, U> for Range<T> {
    type Out = Range<U>;

    fn map_bounds(self, map: impl Fn(T) -> U) -> Self::Out {
        map(self.start)..map(self.end)
    }
}

impl<T, U> MapBounds<T, U> for RangeFrom<T> {
    type Out = RangeFrom<U>;

    fn map_bounds(self, map: impl Fn(T) -> U) -> Self::Out {
        map(self.start)..
    }
}

impl<T, U> MapBounds<T, U> for RangeTo<T> {
    type Out = RangeTo<U>;

    fn map_bounds(self, map: impl Fn(T) -> U) -> Self::Out {
        ..map(self.end)
    }
}

impl<T, U> MapBounds<T, U> for RangeFull {
    type Out = RangeFull;

    fn map_bounds(self, _: impl Fn(T) -> U) -> Self::Out {
        ..
    }
}

pub fn mirror_string(string: &str) -> String {
    let mut graphemes = string.graphemes().collect::<Vec<_>>();
    graphemes.reverse();

    graphemes
        .into_iter()
        .map(|g| {
            match g.as_str() {
                "(" => ")",
                ")" => "(",
                "[" => "]",
                "]" => "[",
                "{" => "}",
                "}" => "{",
                "<" => ">",
                ">" => "<",
                other => other,
            }
            .to_owned()
        })
        .collect()
}

pub fn indent_string(columns: Ix<Column>) -> String {
    unsafe { String::from_utf8_unchecked(vec![b' '; columns.inner()]) }
}

#[extension_trait]
pub impl<T: Ord + Copy> RangeOverlap for Range<T> {
    fn overlaps(&self, other: impl Borrow<Self>) -> bool {
        let other = other.borrow();
        self.start.max(other.start) <= self.end.min(other.end)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CharClass {
    Lower,
    Cap,
    Caseless,
    Number,
    Symbol(char),
}

impl CharClass {
    pub fn of(char: char) -> Self {
        if char.is_lowercase() {
            CharClass::Lower
        } else if char.is_uppercase() {
            CharClass::Cap
        } else if char.is_alphabetic() {
            CharClass::Caseless
        } else if char.is_numeric() {
            CharClass::Number
        } else {
            CharClass::Symbol(char)
        }
    }
}

#[extension_trait]
pub impl LinesColumnsExt for str {
    fn lines_columns(&self) -> (Ix<Line>, Ix<Column>) {
        let lines = Ix::new(self.chars().filter(|&c| c == '\n').count());
        let columns = if !self.ends_with("\n")
            && let Some(line) = self.lines().next_back()
        {
            line.graphemes().map(|g| g.columns()).sum()
        } else {
            Ix::new(0)
        };
        (lines, columns)
    }
}

pub fn auto_removal_char(left: &str) -> Option<&'static str> {
    Some(match left {
        "(" => ")",
        "[" => "]",
        "{" => "}",
        "<" => ">",
        "\"" => "\"",
        "'" => "'",
        "|" => "|",
        _ => None::<!>?,
    })
}

pub fn is_right_delimiter(delimiter: &str) -> bool {
    matches!(delimiter, ")" | "]" | "}" | ">")
}