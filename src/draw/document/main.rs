use std::{cmp::Ordering::*, collections::BTreeMap, iter};

use crate::{
    color,
    custom_literal::integer::rgb,
    document::Document,
    draw::{cursor::{CursorRangeShape, CursorStyle}, document::{highlight::Highlight, query::query_cx}, screen::Canvas},
    grapheme::{Grapheme, GraphemeExt},
    ix::{Byte, Column, Ix, Line, ix},
    style::{Style, Under},
    theme::theme,
};

use super::{super::screen::Cell, CursorRange};

fn resolve_highlight(scopes: &[Highlight], pos: Ix<Byte>) -> Style {
    let scopes = scopes
        .iter()
        .filter(|Highlight { range, .. }| range.contains(&pos))
        .collect::<Vec<_>>();

    let max_injection_layer = scopes
        .iter().copied()
        .filter_map(|&Highlight { injection_layer, .. }| injection_layer)
        .max()
        .unwrap_or_default();

    let mut map = BTreeMap::<i32, Vec<_>>::new();

    scopes
        .iter()
        .filter(|Highlight { injection_layer, .. }|
            injection_layer.is_none_or(|layer| layer == max_injection_layer)
        )
        .for_each(|Highlight { scope, priority, .. }| {

            map.entry(*priority).or_default().push(scope.0.iter().map(|s| &**s).collect::<Vec<_>>());
        });

    let mut style = Style::fg(color::FG) + Style::bg(color::BG);

    for (_, scope) in map {
        style =
            style
            + theme().highlight(&sulu::Highlight::from_iterators(scope));
    }

    style
}

impl Document {
    pub fn main_draw(
        &self,
        mut canvas: Canvas<'_>,
        cursors: impl Fn(Ix<Line>) -> Vec<CursorRange>,
    ) {
        let (width, height) = canvas.size();
        *self.view_height.lock() = ix(height as _);

        let qcx = query_cx!(self);

        let highlight_scopes = self.highlight(&qcx);

        let numbered_lines = self.text().max_numbered_line();
        let gutter_width = if numbered_lines != ix(0) {
            numbered_lines.inner().ilog10() as u16 + 1
        } else {
            0
        };
        let write_line_nr = {
            let width = gutter_width.into();
            move |canvas: &mut Canvas<'_>, line_nr: Ix<Line>, screen_line_nr: u16| {
                let (nr, bg) = if line_nr < numbered_lines {
                    (
                        format!("{:>1$}", line_nr.inner() + 1, width),
                        rgb!(0x301010),
                    )
                } else {
                    (iter::repeat_n(" ", width).collect(), rgb!(0x100000))
                };
                for (j, grapheme) in (0..).into_iter().zip(nr.graphemes()) {
                    canvas[(screen_line_nr, j)] = Cell {
                        is_background: false,
                        grapheme,
                        style: (Style::fg(rgb!(0x604040)) + Style::bg(bg)).into(),
                    };
                }
            }
        };
        let scroll = self.scroll;

        let mut shadow_len = 0u16;
        let mut i = 0;
        for line in self.lines_to(ix(height as _)) {
            shadow_len = shadow_len.saturating_sub(1);
            let gi = ix(i as _) + scroll;
            let line_byte = self.text().byte_of_line(gi).unwrap();
            write_line_nr(&mut canvas, gi, i);

            let len = {
                let mut j = gutter_width;
                for (byte, grapheme) in line.columns_with_bytes().skip(self.horizontal_scroll.inner()) {
                    if j >= width {
                        break;
                    }

                    let hl_style = resolve_highlight(&highlight_scopes, byte + line_byte);

                    canvas[(i, j)] = Cell {
                        is_background: false,
                        grapheme: if let Some(g) = grapheme && !g.is_whitespace() { g } else {Grapheme::SPACE},
                        style: hl_style.into(),
                    };

                    j += 1;
                }
                j
            };

            let inline_diagnostic =
                self.last_line_diagnostic(ix(i as _) + scroll)
                    .map(|(s, m)| {
                        (
                            s,
                            m.graphemes()
                                .take_while(|p| !p.is_newline())
                                .collect::<Vec<_>>(),
                        )
                    });

            if width > len {
                for (rj, j) in (len..width).into_iter().enumerate() {
                    let cell = &mut canvas[(i, j)];

                    match j.cmp(&shadow_len) {
                        Less => cell.style.bg = color::SHADOW,
                        Equal => {
                            cell.style.fg = color::SHADOW;
                            cell.grapheme = Grapheme::UPPER_LEFT_TRIANGLE;
                            cell.is_background = true;
                        }
                        Greater => (),
                    }
                    if let Some((severity, message)) = &inline_diagnostic {
                        const MESSAGE_GAP: usize = 2;
                        if rj < MESSAGE_GAP {
                            continue;
                        }
                        if rj == MESSAGE_GAP {
                            cell.style.fg = severity.bg();
                            cell.grapheme = Grapheme::LEFT_TRIANGLE;
                        } else if rj < message.len() + MESSAGE_GAP + 1 {
                            cell.style.fg = severity.fg();
                            cell.style.bg = severity.bg();
                            cell.style.italic = true;
                            let grapheme = message[rj - MESSAGE_GAP - 1].clone();
                            if grapheme.is_whitespace() {
                                cell.grapheme = Grapheme::SPACE;
                            } else {
                                cell.grapheme = grapheme;
                            }
                        } else if rj == message.len() + MESSAGE_GAP + 1 {
                            cell.style.fg = severity.bg();
                            cell.grapheme = Grapheme::RIGHT_TRIANGLE;
                        }
                    }
                }
            }

            shadow_len = shadow_len.max(len);

            i += 1;
        }

        while i < height {
            shadow_len = shadow_len.saturating_sub(1);
            let gi = ix(i as usize) + scroll;
            write_line_nr(&mut canvas, gi, i);
            for j in gutter_width..width {
                let cell = &mut canvas[(i, j)];
                match j.cmp(&shadow_len) {
                    Less => cell.style.bg = color::SHADOW,
                    Equal => {
                        cell.style.fg = color::SHADOW;
                        cell.grapheme = Grapheme::UPPER_LEFT_TRIANGLE;
                        cell.is_background = true;
                    }
                    Greater => (),
                }
            }

            i += 1;
        }
    }
}
