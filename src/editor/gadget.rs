use crossterm::event::KeyEvent;

use crate::{draw::screen::Canvas, editor::Editor};

pub enum ScreenRegion {
    RightPanel,
    DocOverlay,
}

pub trait Gadget {
    fn on_key(&mut self, #[allow(unused)] event: KeyEvent) -> Option<Box<dyn FnOnce(&mut Editor)>> {
        None
    }

    fn screen_region(&self) -> ScreenRegion {
        ScreenRegion::RightPanel
    }

    fn draw(&self, #[allow(unused)] canvas: Canvas<'_>) {}
}
