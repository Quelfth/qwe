use std::{borrow::Borrow, fmt::Write, ops::Range};

use extension_trait::extension_trait;
use tree_sitter::Node;

use crate::{
    grapheme::GraphemeExt,
    ix::{Column, Ix},
};

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

#[extension_trait]
pub impl<T> MapBounds for Range<T> {
    type T = T;

    fn map_bounds<U>(self, map: impl Fn(Self::T) -> U) -> Range<U> {
        map(self.start)..map(self.end)
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
