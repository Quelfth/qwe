use crate::{draw::screen::Canvas, editor::Editor, key::KeyOrChar};

pub enum ScreenRegion {
    RightPanel,
    DocOverlay,
}

pub trait Gadget {
    fn on_key(&mut self, event: KeyOrChar) -> Option<Box<dyn FnOnce(&mut Editor)>> {
        _= event;
        None
    }

    fn screen_region(&self) -> ScreenRegion {
        ScreenRegion::RightPanel
    }

    fn draw(&self, #[allow(unused)] canvas: Canvas<'_>) {}
}
