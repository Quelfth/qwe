use std::{
    io::{self},
    iter,
};

use auto_enums::auto_enum;
use crossterm::style::Color;
use culit::culit;

use crate::{
    custom_literal::integer::rgb,
    draw::screen::{Cell, Screen},
    editor::{
        Editor,
        cursors::{CursorState, select::RangeCursorLine},
    },
    grapheme::GraphemeExt,
    terminal_size::terminal_size,
};

pub mod screen;

#[derive(Copy, Clone)]
struct CursorRange {
    kind: CursorRangeKind,
    range: Range,
}

impl CursorRange {
    fn thin(
        pos: usize,
        left: CursorRangeKind,
        right: CursorRangeKind,
    ) -> impl Iterator<Item = Self> {
        [
            (pos > 0).then(|| Self {
                kind: left,
                range: Range::one(pos - 1),
            }),
            Some(Self {
                kind: right,
                range: Range::one(pos),
            }),
        ]
        .into_iter()
        .flatten()
    }

    fn insert(pos: usize) -> impl Iterator<Item = Self> {
        Self::thin(
            pos,
            CursorRangeKind::InsertLeft,
            CursorRangeKind::InsertRight,
        )
    }

    #[auto_enum(Iterator)]
    fn select(start: usize, end: usize) -> impl Iterator<Item = Self> {
        match start == end {
            true => Self::thin(
                start,
                CursorRangeKind::SelectLeft,
                CursorRangeKind::SelectRight,
            ),
            false => iter::once(Self {
                kind: CursorRangeKind::Select,
                range: Range { start, end },
            }),
        }
    }
}

#[derive(Copy, Clone)]
enum CursorRangeKind {
    InsertLeft,
    InsertRight,
    Select,
    SelectLeft,
    SelectRight,
}

impl CursorRangeKind {
    #[culit]
    fn color(self) -> Color {
        match self {
            CursorRangeKind::InsertLeft => 0x003830rgb,
            CursorRangeKind::InsertRight => 0x007060rgb,
            CursorRangeKind::Select => 0x202070rgb,
            CursorRangeKind::SelectLeft => 0x101050rgb,
            CursorRangeKind::SelectRight => 0x404090rgb,
        }
    }
}

#[derive(Copy, Clone)]
struct Range {
    start: usize,
    end: usize,
}

impl Range {
    fn one(pos: usize) -> Self {
        Self {
            start: pos,
            end: pos + 1,
        }
    }

    fn contains(self, pos: usize) -> bool {
        pos >= self.start && pos < self.end
    }
}

impl CursorState {
    #[auto_enum(Iterator)]
    fn line_ranges(&self, line: usize) -> impl Iterator<Item = CursorRange> {
        match self {
            CursorState::Insert(cursors) => cursors
                .iter()
                .flat_map(move |c| (c.pos.line == line).then(|| CursorRange::insert(c.pos.column)))
                .flatten(),
            CursorState::Select(cursors) => cursors
                .iter()
                .filter_map(move |c| {
                    let RangeCursorLine { start, end } = c.on_line(line)?;
                    Some(CursorRange::select(start, end))
                })
                .flatten(),
            _ => iter::empty(),
        }
    }
}

impl Editor {
    pub fn draw(&self) -> io::Result<()> {
        let (width, height) = terminal_size();
        let mut screen = Screen::new(width, height);

        fn cursor_color(cursors: &[CursorRange]) -> impl Fn(usize) -> Option<Color> {
            |i| {
                cursors
                    .iter()
                    .find(|c| c.range.contains(i))
                    .map(|c| c.kind)
                    .map(|k| k.color())
            }
        }

        let numbered_lines = self.doc().text().line_count() + 1;
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
                        fg: rgb!(0x604040),
                        bg,
                    };
                }
            }
        };
        let scroll = self.doc().scroll;

        let mut i = 0;
        for line in self.doc().lines_to(height as _) {
            let gi = i as usize + scroll;
            let cursors = self.cursors().line_ranges(gi).collect::<Vec<_>>();
            let cursor_color = cursor_color(&cursors);

            let len = {
                write_line_nr(&mut screen, gi, i);
                let mut j = gutter_width;
                for grapheme in line.graphemes() {
                    if j >= width - gutter_width {
                        break;
                    }
                    screen[(i, j)] = Cell {
                        grapheme,
                        fg: rgb! {0xffffff},
                        bg: cursor_color((j - gutter_width) as usize).unwrap_or(rgb! {0x200000}),
                    };

                    j += 1;
                }
                j
            };

            if width > len {
                for j in len..width {
                    if let Some(color) = cursor_color((j - gutter_width) as usize) {
                        screen[(i, j)].bg = color;
                    }
                }
            }

            i += 1;
        }

        while i < height {
            let gi = i as usize + scroll;
            let cursors = self.cursors().line_ranges(gi).collect::<Vec<_>>();
            let cursor_color = cursor_color(&cursors);
            write_line_nr(&mut screen, gi, i);
            for j in gutter_width..width {
                if let Some(color) = cursor_color((j - gutter_width) as usize) {
                    screen[(i, j)].bg = color;
                }
            }

            i += 1;
        }

        {
            let last_screen = &mut *self.screen.lock();
            screen.draw_diff(last_screen)?;
            *last_screen = screen;
        }

        Ok(())
    }
}
