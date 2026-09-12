use crate::{
    document::Document,
    draw::{cursor::{CursorRangeShape, CursorStyle}, screen::Canvas},
    style::{Style, Under},
};


impl Document {
    pub fn draw_cursors(&self, mut canvas: Canvas<'_>) {
        let Some(cursors) = &self.cursors else {return};
        let gw = self.gutter_width();

        for cursor in cursors.ranges() {
            let style = match cursor.r#type.style() {
                CursorStyle::Color(color) => Style::bg(color),
                CursorStyle::Underline(color) => Style::from(Under::Line) + Style::uc(Some(color)),
            };
            match cursor.range {
                CursorRangeShape::Range { line, range } => {
                    let line = line.saturating_sub(self.scroll).0 as u16;
                    let start = gw + range.start.saturating_sub(self.horizontal_scroll).0 as u16;
                    let end = gw + range.end.saturating_sub(self.horizontal_scroll).0 as u16;
                    canvas.overlay_rect(line..=line, start..end, style);
                },
                CursorRangeShape::Line(range) => {
                    let start = range.start.saturating_sub(self.scroll).0 as u16;
                    let end = range.end.saturating_sub(self.scroll).0 as u16;
                    canvas.overlay_rect(start..end, .., style);
                },
            }
        }
    }
}
