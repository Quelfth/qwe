use std::fmt::{Display, Write};

use tree_sitter::Node;

pub fn default<T: Default>() -> T {
    T::default()
}

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
