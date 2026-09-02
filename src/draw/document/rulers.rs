use std::collections::HashMap;

use tree_sitter::{Node, QueryCursor};

use crate::{
    custom_literal::integer::rgb,
    document::{Document, tree::MetaQueryCapture},
    draw::screen::Canvas,
    grapheme::Grapheme,
    ix::{Ix, Line, ix},
    lang::Rulers,
    theme::theme,
};


impl Document {
    pub fn draw_rulers(&self, mut canvas: Canvas<'_>) {
        let mut indent = ix(0);
        let g = self.gutter_width();
        let mut paragraph_start = self.scroll;
        while self.text().context_indent_inc(paragraph_start) != ix(0) && let Some(p) = paragraph_start.checked_sub(ix(1)) {
            paragraph_start = p;
        }

        let mut cx = self.query_cx();
        cx.relevant_lines.start = paragraph_start;

        let mut rulers = HashMap::<Node, Vec<&'static str>>::new();

        if let Some(tree) = self.tree() && let Some(lang) = self.language() {
            for MetaQueryCapture { node, name, .. } in tree.query::<Rulers>(&cx, &mut QueryCursor::new(), self.text(), lang) {
                rulers.entry(node).or_default().push(name);
            }
        }

        for l in paragraph_start..self.scroll + ix(canvas.height() as _) {
            let new_indent = self.text().context_indent_inc(l);
            if new_indent > indent {
                let mut j = l;
                while self.text().context_indent_inc(j) >= new_indent && j <= self.text().line_len() {
                    j += ix(1);
                }
                while j > l && !self.text().line_has_content(j - ix(1)) {
                    j -= ix(1);
                }
                let Some(c) = (try { indent.checked_sub(self.horizontal_scroll)?.inner() as u16 + g }) else {
                    indent = new_indent;
                    continue;
                };
                if c >= canvas.width() {
                    indent = new_indent;
                    continue;
                }

                let range = self.text().byte_of_line(l).unwrap_or(self.text().byte_len())..self.text().byte_of_line(j).unwrap_or(self.text().byte_len());
                let node = if let Some(tree) = self.tree() {
                    tree.tree.root_node().descendant_for_byte_range(range.start.inner(), range.end.inner())
                } else {
                    None
                };

                let hl = node.and_then(|n| {
                    let scopes = rulers.get(&n)?;
                    let highlight = sulu::Highlight::<&'static str>::from_iterators(scopes.iter().map(|scope| scope.split('.')));
                    let hl = theme().highlight(&highlight);
                    Some(hl)
                });

                let ux = |i: Ix<Line>| {
                    i.saturating_sub(self.scroll).inner() as u16
                };
                let j = ux(j);
                for r in ux(l)..j {
                    if r >= canvas.height() {break}
                    let cell = &mut canvas[(r, c)];
                    cell.grapheme = match (ix(r as usize) + self.scroll == l, r + 1 == j) {
                        (true, true) => Grapheme::BULLET,
                        (true, false) => Grapheme::RIGHT_PAREN_TOP,
                        (false, true) => Grapheme::RIGHT_PAREN_BOTTOM,
                        (false, false) => Grapheme::RIGHT_PAREN_MIDDLE,
                    };
                    if let Some(hl) = hl && let Some(fg) = hl.fg {
                        cell.style.fg = fg;
                    } else {
                        cell.style.fg = rgb!(0x301010);
                    }

                }
            }
            indent = new_indent;
        }
    }
}
