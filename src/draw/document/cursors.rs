use crate::{
    color,
    document::Document,
    draw::{cursor::{CursorElementShape, CursorStyle}, screen::{Canvas, ScreenCursor}},
    style::{Style, Under},
};


impl Document {
    pub fn draw_cursors(&self, mut canvas: Canvas<'_>) {
        let Some(cursors) = &self.cursors else {return};
        let gw = self.gutter_width();

        for cursor in cursors.elements() {
            let style = match cursor.r#type.style() {
                CursorStyle::Color(color) => Style::bg(color),
                CursorStyle::Underline(color) => Style::from(Under::Line) + Style::uc(Some(color)),
            };
            try {
                match cursor.range {
                    CursorElementShape::Range { line, range } => {
                        let line = line.checked_sub(self.scroll)?.0 as u16;
                        let start = gw + range.start.saturating_sub(self.horizontal_scroll).0 as u16;
                        let end = gw + range.end.checked_sub(self.horizontal_scroll)?.0 as u16;
                        canvas.overlay_rect(line..=line, start..end, style);
                    },
                    CursorElementShape::Line(range) => {
                        let start = range.start.saturating_sub(self.scroll).0 as u16;
                        let end = range.end.checked_sub(self.scroll)?.0 as u16;
                        canvas.overlay_rect(start..end, .., style);
                    },
                    CursorElementShape::Caret(pos) => {
                        let line = pos.line.checked_sub(self.scroll)?.0 as u16;
                        let col = gw + pos.column.checked_sub(self.horizontal_scroll)?.0 as u16;
                        canvas.draw_cursor(ScreenCursor { position: (line, col), color: style.bg.unwrap_or(color::FG) });
                    }
                }
            };
        }
    }
}
