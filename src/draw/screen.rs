use std::{
    io::{self, Write, stdout},
    ops::{Index, IndexMut},
};

use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    style::{Color, ContentStyle, PrintStyledContent, StyledContent, Stylize},
};
use culit::culit;

use crate::{grapheme::Grapheme, style::FlatStyle};

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
