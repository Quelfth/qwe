use crate::{
    document::Document,
    draw::screen::Canvas,
};

pub mod badges;
pub mod highlight;
pub mod main;
pub mod query;
pub mod locals;
pub mod rulers;
pub mod annotations;
pub mod cursors;

impl Document {
    pub fn draw(&self, mut canvas: Canvas<'_>) {
        self.main_draw(canvas.reborrow());
        self.draw_cursors(canvas.reborrow());
        self.draw_rulers(canvas.reborrow());
        self.draw_annotations(canvas.reborrow());
        self.draw_edge_indicators(canvas);
    }
}
