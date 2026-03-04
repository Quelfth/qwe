use std::{
    io::{self, Write, stdout},
    ops::{Index, IndexMut},
};

use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    style::{ContentStyle, PrintStyledContent, StyledContent},
};
use culit::culit;

use crate::{
    draw::{Range, Rect},
    grapheme::Grapheme,
    style::FlatStyle,
};

#[derive(Default)]
pub struct Screen {
    width: u16,
    height: u16,
    cells: Box<[Cell]>,
}

impl Screen {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::default(); (width * height).into()].into(),
        }
    }
}

impl Index<(u16, u16)> for Screen {
    type Output = Cell;

    fn index(&self, (row, col): (u16, u16)) -> &Self::Output {
        &self.cells[(row * self.width + col) as usize]
    }
}

impl IndexMut<(u16, u16)> for Screen {
    fn index_mut(&mut self, (row, col): (u16, u16)) -> &mut Self::Output {
        &mut self.cells[(row * self.width + col) as usize]
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Cell {
    pub grapheme: Grapheme,
    pub style: FlatStyle,
}

impl Default for Cell {
    #[culit]
    fn default() -> Self {
        Self {
            grapheme: Default::default(),
            style: FlatStyle {
                fg: 0x604040rgb,
                bg: 0x100000rgb,
                ..Default::default()
            },
        }
    }
}

impl From<Cell> for StyledContent<Grapheme> {
    fn from(value: Cell) -> Self {
        StyledContent::new(value.style(), value.grapheme)
    }
}

impl Cell {
    fn style(&self) -> ContentStyle {
        self.style.into()
    }

    fn as_styled(&self) -> StyledContent<&str> {
        StyledContent::new(self.style(), self.grapheme.as_str())
    }
}

impl Screen {
    pub fn draw_full(&self) -> io::Result<()> {
        let mut stdout = stdout();
        for i in 0..self.height {
            stdout.queue(MoveTo(0, i))?;
            for j in 0..self.width {
                let cell = &self[(i, j)];
                stdout.queue(PrintStyledContent(cell.as_styled()))?;
            }
        }

        stdout.flush()
    }

    pub fn draw_diff(&self, prev: &Self) -> io::Result<()> {
        if self.width != prev.width || self.height != prev.height {
            return self.draw_full();
        }
        let mut stdout = stdout();
        for i in 0..self.height {
            for j in 0..self.width {
                if prev[(i, j)] == self[(i, j)] {
                    continue;
                }
                let cell = &self[(i, j)];
                stdout
                    .queue(MoveTo(j, i))?
                    .queue(PrintStyledContent(cell.as_styled()))?;
            }
        }

        stdout.flush()
    }
}

impl Screen {
    pub fn canvas(&mut self, rect: Rect<u16>) -> Canvas<'_> {
        Canvas { screen: self, rect }
    }
}

pub struct Canvas<'a> {
    screen: &'a mut Screen,
    rect: Rect<u16>,
}

impl<'s> Canvas<'s> {
    pub fn width(&self) -> u16 {
        self.rect.width()
    }

    pub fn height(&self) -> u16 {
        self.rect.height()
    }

    pub fn size(&self) -> (u16, u16) {
        (self.width(), self.height())
    }

    pub fn take_top(&mut self, amount: u16) -> Canvas<'_> {
        let Canvas {
            screen,
            rect: Rect { rows, cols },
        } = self;
        Canvas {
            screen,
            rect: Rect {
                rows: Range {
                    start: rows.start,
                    end: (rows.start + amount).min(rows.end),
                },
                cols: *cols,
            },
        }
    }

    pub fn shrink_top(&mut self, by: u16) -> Canvas<'_> {
        let Canvas {
            screen,
            rect: Rect { rows, cols },
        } = self;
        Canvas {
            screen,
            rect: Rect {
                rows: Range {
                    start: (rows.start + by).min(rows.end),
                    end: rows.end,
                },
                cols: *cols,
            },
        }
    }

    pub fn reborrow<'a>(&'a mut self) -> Canvas<'a> {
        Canvas {
            screen: self.screen,
            rect: self.rect,
        }
    }
}

impl IndexMut<(u16, u16)> for Canvas<'_> {
    fn index_mut(&mut self, (i, j): (u16, u16)) -> &mut Self::Output {
        let (width, height) = (self.rect.width(), self.rect.height());
        if i > self.rect.height() {
            panic!("row {i} is out of bounds for canvas of height {height}")
        }
        if j > self.rect.width() {
            panic!("column {j} is out of bounds for canvas of width {width}")
        }
        &mut self.screen[(i + self.rect.rows.start, j + self.rect.cols.start)]
    }
}

impl Index<(u16, u16)> for Canvas<'_> {
    type Output = Cell;

    fn index(&self, (i, j): (u16, u16)) -> &Self::Output {
        let (width, height) = (self.rect.width(), self.rect.height());
        if i > self.rect.height() {
            panic!("row {i} is out of bounds for canvas of height {height}")
        }
        if j > self.rect.width() {
            panic!("column {j} is out of bounds for canvas of width {width}")
        }
        &self.screen[(i + self.rect.rows.start, j + self.rect.cols.start)]
    }
}
