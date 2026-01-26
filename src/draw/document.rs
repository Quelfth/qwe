use std::{cmp::Ordering::*, io, iter};

use crossterm::style::Color;
use tree_sitter::{QueryCapture, QueryCursor, QueryMatch, StreamingIterator};

use crate::{
    custom_literal::integer::rgb,
    document::Document,
    grapheme::{Grapheme, GraphemeExt},
    style::Style,
    theme::theme,
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
    ) {
        let y = rect.rows.start;
        let x = rect.cols.start;
        let width = rect.width();
        let height = rect.height();

        fn cursor_color(cursors: &[CursorRange]) -> impl Fn(usize) -> Option<Color> {
            |i| {
                cursors
                    .iter()
                    .find(|c| c.range.is_none_or(|r| r.contains(i)))
                    .map(|c| c.kind)
                    .map(|k| k.color())
            }
        }

        let mut highlight_scopes = Vec::new();

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

            while let Some(QueryMatch {
                pattern_index: _,
                captures,
                ..
            }) = matches.next()
            {
                for QueryCapture { node, index } in *captures {
                    let name = query.capture_names()[*index as usize];
                    highlight_scopes.push((
                        name.split(".").map(|s| s.to_owned()).collect::<Vec<_>>(),
                        node.byte_range(),
                    ));
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
                    screen[(screen_line_nr + y, j + x)] = Cell {
                        grapheme,
                        style: (Style::fg(rgb!(0x604040)) + Style::bg(bg)).into(),
                    };
                }
            }
        };
        let scroll = self.scroll;

        let mut shadow_len = 0u16;
        let mut i = 0;
        for line in self.lines_to(height as _) {
            shadow_len = shadow_len.saturating_sub(1);
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
                    let hl_scopes = highlight_scopes
                        .iter()
                        .filter(|(_, r)| r.contains(&(byte + line_byte)))
                        .map(|(s, _)| s.iter().map(|s| &**s).collect::<Vec<_>>())
                        .collect::<Vec<_>>();
                    let hl_style = (Style::fg(rgb! {0xcca4a4}) + Style::bg(rgb! {0x200000}))
                        + theme().highlight(&hl_scopes);
                    screen[(i + y, j + x)] = Cell {
                        grapheme,
                        style: (hl_style
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
                    let cell = &mut screen[(i + y, j + x)];
                    if let Some(color) = cursor_color((j - gutter_width) as usize) {
                        cell.style.bg = color;
                    } else {
                        match j.cmp(&shadow_len) {
                            Less => cell.style.bg = rgb! {0x160000},
                            Equal => {
                                cell.style.fg = rgb! {0x160000};
                                cell.grapheme = Grapheme::UPPER_LEFT_TRIANGLE;
                            }
                            Greater => (),
                        }
                    }
                }
            }

            shadow_len = shadow_len.max(len);

            i += 1;
        }

        while i < height {
            let gi = i as usize + scroll;
            let cursors = cursors(gi);
            let cursor_color = cursor_color(&cursors);
            write_line_nr(screen, gi, i);
            for j in gutter_width..width {
                if let Some(color) = cursor_color((j - gutter_width) as usize) {
                    screen[(i + y, j + x)].style.bg = color;
                }
            }

            i += 1;
        }
    }
}
