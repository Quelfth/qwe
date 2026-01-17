use std::{io, iter};

use crossterm::style::Color;
use tree_sitter::{QueryCapture, QueryCursor, QueryMatch, StreamingIterator};

use crate::{
    aprintln::aprintln,
    custom_literal::integer::rgb,
    document::Document,
    grapheme::GraphemeExt,
    style::{FlatStyle, Style},
};

use super::{
    CursorRange, Rect,
    screen::{Cell, Screen},
};

impl Document {
    pub(super) fn draw(
        &self,
        screen: &mut Screen,
        rect: Rect<u16>,
        cursors: impl Fn(usize) -> Vec<CursorRange>,
    ) -> io::Result<()> {
        let width = rect.width();
        let height = rect.height();

        fn cursor_color(cursors: &[CursorRange]) -> impl Fn(usize) -> Option<Color> {
            |i| {
                cursors
                    .iter()
                    .find(|c| c.range.contains(i))
                    .map(|c| c.kind)
                    .map(|k| k.color())
            }
        }

        let mut highlights = Vec::new();

        if let (Some(lang), Some(tree)) = (self.language(), self.tree()) {
            let mut cursor = QueryCursor::new();
            let root = tree.root_node();

            let query = lang.highlight_query_source().build().unwrap();

            let mut matches = cursor.matches_with_options(
                &query,
                root,
                self.text(),
                tree_sitter::QueryCursorOptions {
                    progress_callback: None,
                },
            );

            while let Some(r#match) = matches.next() {
                let QueryMatch {
                    pattern_index: _,
                    captures,
                    ..
                } = r#match;

                for QueryCapture { node, index } in *captures {
                    let name = query.capture_names()[*index as usize];
                    if name == "i" {
                        highlights.push((Style::fg(rgb! {0xff0000}), node.byte_range()))
                    }
                    //aprintln!("{name}: {}", node.to_sexp());
                }
            }
        }

        let numbered_lines = self.text().line_count() + 1;
        let gutter_width = numbered_lines.ilog10() as u16 + 1;
        let write_line_nr = {
            let width = gutter_width.into();
            move |screen: &mut Screen, line_nr: usize, screen_line_nr: u16| {
                let (nr, bg) = if line_nr < numbered_lines {
                    (format!("{:>1$}", line_nr + 1, width), rgb!(0x301010))
                } else {
                    (iter::repeat_n(" ", width).collect(), rgb!(0x100000))
                };
                for (j, grapheme) in (0..).zip(nr.graphemes()) {
                    screen[(screen_line_nr, j)] = Cell {
                        grapheme,
                        style: (Style::fg(rgb!(0x604040)) + Style::bg(bg)).into(),
                    };
                }
            }
        };
        let scroll = self.scroll;

        let mut i = 0;
        for line in self.lines_to(height as _) {
            let gi = i as usize + scroll;
            let line_byte = self.text().byte_of_line(gi).unwrap();
            let cursors = cursors(gi);
            let cursor_color = cursor_color(&cursors);

            let len = {
                write_line_nr(screen, gi, i);
                let mut j = gutter_width;
                for (byte, grapheme) in line.graphemes_with_bytes() {
                    if j >= width - gutter_width {
                        break;
                    }
                    let highlight_style = highlights
                        .iter()
                        .filter(|(_, r)| r.contains(&(byte + line_byte)))
                        .map(|(s, _)| *s)
                        .fold(
                            Style::fg(rgb! {0xffffff}) + Style::bg(rgb! {0x200000}),
                            |c, n| c + n,
                        );
                    screen[(i, j)] = Cell {
                        grapheme,
                        style: (highlight_style
                            + cursor_color((j - gutter_width) as usize)
                                .map(Style::bg)
                                .unwrap_or_default())
                        .into(),
                    };

                    j += 1;
                }
                j
            };

            if width > len {
                for j in len..width {
                    if let Some(color) = cursor_color((j - gutter_width) as usize) {
                        screen[(i, j)].style.bg = color;
                    }
                }
            }

            i += 1;
        }

        while i < height {
            let gi = i as usize + scroll;
            let cursors = cursors(gi);
            let cursor_color = cursor_color(&cursors);
            write_line_nr(screen, gi, i);
            for j in gutter_width..width {
                if let Some(color) = cursor_color((j - gutter_width) as usize) {
                    screen[(i, j)].style.bg = color;
                }
            }

            i += 1;
        }

        Ok(())
    }
}
