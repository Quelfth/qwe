use crate::{
    document::Document,
    draw::screen::Canvas,
};

use super::CursorRange;

pub mod badges;
pub mod highlight;
pub mod main;
pub mod query;

impl Document {
    pub fn draw(&self, mut canvas: Canvas<'_>) {
        let cursors = |i| {
            self.cursors
                .as_ref()
                .map(|c| c.ranges_for_line(i).collect())
                .unwrap_or_default()
        };
        self.main_draw(canvas.reborrow(), cursors);
        self.draw_edge_indicators(canvas);
    }
}
