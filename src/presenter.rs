use std::{cell::Cell, io, time::{Duration, Instant}};

use crossterm::style::Color;
use dispa::dispatch;
use mutx::Mutex;

use crate::{color, draw::{Rect, screen::{Canvas, Screen}}, terminal_size::terminal_size};

#[derive(Default)]
pub struct Presenter {
    screen: Mutex<Screen>,
    last_draw: Cell<Option<Instant>>,
    draw_defer: Cell<Option<Instant>>,
}

impl Presenter {
    pub fn defer_draw(&self) {
        const DEFER_DURATION: Duration = Duration::from_millis(50);
        self.defer_draw_to(Instant::now() + DEFER_DURATION);
    }

    fn defer_draw_to(&self, instant: Instant) {
        if self.draw_defer.get().is_none() {
            self.draw_defer.set(Some(instant));
        }
    }

    pub fn draw(&self, layout: &(impl Present + ?Sized)) -> io::Result<()> {
        const MIN_REDRAW: Duration = Duration::from_millis(8);
        if let Some(i) = self.last_draw.get() && i.elapsed() < MIN_REDRAW {
            self.defer_draw_to(i + MIN_REDRAW);
            return Ok(());
        }

        let (width, height) = terminal_size();
        let mut screen = Screen::new(width, height, layout.bg_color());

        _ = layout.present(screen.canvas(Rect { rows: (0..height).into(), cols: (0..width).into() }));

        {
            let last_screen = &mut *self.screen.lock();
            screen.draw_diff(last_screen)?;
            *last_screen = screen;
        }

        self.draw_defer.set(None);
        self.last_draw.set(Some(Instant::now()));
        Ok(())
    }
}

#[dispatch]
pub trait Present {
    fn present(&self, canvas: Canvas<'_>) -> io::Result<()>;

    fn presenter(&self) -> &Presenter;

    fn bg_color(&self) -> Color { color::DEEP_BG }

    fn draw(&self) -> io::Result<()> {
        self.presenter().draw(self)
    }
    
    fn poll_draw(&self) -> io::Result<()> {
        if let Some(defer) = self.presenter().draw_defer.get() && defer <= Instant::now() {
            self.draw()?;
        }
        Ok(())
    }
}