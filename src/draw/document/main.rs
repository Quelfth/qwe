use std::{cmp::Ordering::*, iter};

use crate::{
    color,
    custom_literal::integer::rgb,
    document::Document,
    draw::{cursor::CursorStyle, document::highlight::Highlight, screen::Canvas},
    grapheme::{Grapheme, GraphemeExt},
    ix::{Column, Ix, Line},
    style::{Style, Under},
    theme::theme,
};

use super::{super::screen::Cell, CursorRange};

impl Document {
    pub fn main_draw(
        &self,
        mut canvas: Canvas<'_>,
        cursors: impl Fn(Ix<Line>) -> Vec<CursorRange>,
    ) {
        let (width, height) = canvas.size();
        *self.view_height.lock() = Ix::new(height as _);

        fn cursor_color(cursors: &[CursorRange]) -> impl Fn(Ix<Column>) -> Option<CursorStyle> {
            |i| {
                cursors
                    .iter()
                    .find(|c| c.range.is_none_or(|r| r.contains(i)))
                    .map(|c| c.r#type)
                    .map(|k| k.style())
            }
        }
        let highlight_scopes = self.highlight();

        let numbered_lines = self.text().max_line_number();
        let gutter_width = if numbered_lines != Ix::new(0) {
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
                for (j, grapheme) in (0..).zip(nr.graphemes()) {
                    canvas[(screen_line_nr, j)] = Cell {
                        grapheme,
                        style: (Style::fg(rgb!(0x604040)) + Style::bg(bg)).into(),
                    };
                }
            }
        };
        let scroll = self.scroll;

        let mut shadow_len = 0u16;
        let mut i = 0;
        for line in self.lines_to(Ix::new(height as _)) {
            shadow_len = shadow_len.saturating_sub(1);
            let gi = Ix::new(i as _) + scroll;
            let line_byte = self.text().byte_of_line(gi).unwrap();
            let cursors = cursors(gi);
            let cursor_color = cursor_color(&cursors);

            let len = {
                write_line_nr(&mut canvas, gi, i);
                let mut j = gutter_width;
                for (byte, grapheme) in line.graphemes_with_bytes().skip(self.horizontal_scroll.inner()) {
                    if j >= width {
                        break;
                    }
                    let hl_scopes = highlight_scopes
                        .iter()
                        .filter(|Highlight { range, .. }| range.contains(&(byte + line_byte)))
                        .map(|Highlight { scope, .. }| {
                            scope.0.iter().map(|s| &**s).collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>();

                    let hl_style = (Style::fg(color::FG) + Style::bg(color::BG))
                        + theme().highlight(&hl_scopes);
                    canvas[(i, j)] = Cell {
                        grapheme,
                        style: {
                            hl_style
                                + cursor_color(Ix::new((j - gutter_width) as _) + self.horizontal_scroll)
                                    .map(|c| match c {
                                        CursorStyle::Color(color) => Style::bg(color),
                                        CursorStyle::Underline(color) => {
                                            Style::uc(Some(color)) + Under::Line.into()
                                        }
                                    })
                                    .unwrap_or_default()
                        }
                        .into(),
                    };

                    j += 1;
                }
                j
            };

            let inline_diagnostic =
                self.last_line_diagnostic(Ix::new(i as _) + scroll)
                    .map(|(s, m)| {
                        (
                            s,
                            m.graphemes()
                                .take_while(|p| !p.is_newline())
                                .collect::<Vec<_>>(),
                        )
                    });

            if width > len {
                for (rj, j) in (len..width).enumerate() {
                    let cell = &mut canvas[(i, j)];
                    if let Some(style) = cursor_color(Ix::new((j - gutter_width) as usize) + self.horizontal_scroll) {
                        use CursorStyle::*;
                        match style {
                            Color(color) => {
                                cell.style.bg = color;
                                continue;
                            }
                            Underline(color) => {
                                cell.style.under = Some(Under::Line);
                                cell.style.uc = Some(color);
                            }
                        }
                    }

                    match j.cmp(&shadow_len) {
                        Less => cell.style.bg = color::SHADOW,
                        Equal => {
                            cell.style.fg = color::SHADOW;
                            cell.grapheme = Grapheme::UPPER_LEFT_TRIANGLE;
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
            let gi = Ix::new(i as usize) + scroll;
            let cursors = cursors(gi);
            let cursor_color = cursor_color(&cursors);
            write_line_nr(&mut canvas, gi, i);
            for j in gutter_width..width {
                let cell = &mut canvas[(i, j)];
                if let Some(style) = cursor_color(Ix::new((j - gutter_width) as usize) + self.horizontal_scroll) {
                    use CursorStyle::*;
                    match style {
                        Color(color) => {
                            cell.style.bg = color;
                            continue;
                        }
                        Underline(color) => {
                            cell.style.under = Some(Under::Line);
                            cell.style.uc = Some(color);
                        }
                    }
                }
                match j.cmp(&shadow_len) {
                    Less => cell.style.bg = color::SHADOW,
                    Equal => {
                        cell.style.fg = color::SHADOW;
                        cell.grapheme = Grapheme::UPPER_LEFT_TRIANGLE;
                    }
                    Greater => (),
                }
            }

            i += 1;
        }
    }
}
