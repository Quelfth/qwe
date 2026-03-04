use crate::{
    custom_literal::integer::rgb,
    document::Document,
    draw::screen::Canvas,
    editor::cursors::Cursors,
    grapheme::{Grapheme, GraphemeExt},
};

impl Document {
    pub fn draw_edge_indicators(&self, mut canvas: Canvas<'_>) {
        let mut above = 0;
        let mut below = 0;

        let screen_range = self.scroll..self.scroll + *self.view_height.lock();

        if let Some(cursors) = &self.cursors {
            for range in cursors.line_ranges() {
                if range.end <= screen_range.start {
                    above += 1;
                }
                if range.start >= screen_range.end {
                    below += 1;
                }
            }
        }

        const VERT_MARGIN: u16 = 1;
        const RIGHT_MARGIN: u16 = 4;
        let fg = rgb!(0x20D0D0);
        let bg = rgb!(0x103050);

        if above > 0 {
            let above_text = format!("▲ {above}");
            let above_right = canvas.width().saturating_sub(RIGHT_MARGIN);
            let above_left = above_right.saturating_sub(above_text.len() as u16);

            let cell = &mut canvas[(VERT_MARGIN, above_left)];
            cell.grapheme = Grapheme::LEFT_SEMICIRCLE;
            cell.style.fg = bg;

            for (i, g) in
                (above_left + 1..above_right.saturating_sub(1)).zip(above_text.graphemes())
            {
                let cell = &mut canvas[(VERT_MARGIN, i)];
                cell.grapheme = g;
                cell.style.fg = fg;
                cell.style.bg = bg;
            }

            let cell = &mut canvas[(VERT_MARGIN, above_right.saturating_sub(1))];
            cell.grapheme = Grapheme::RIGHT_SEMICIRCLE;
            cell.style.fg = bg;
        }
        if below > 0 {
            let below_height = canvas.height().saturating_sub(VERT_MARGIN + 1);
            let below_text = format!("▼ {below}");
            let below_right = canvas.width().saturating_sub(RIGHT_MARGIN);
            let below_left = below_right.saturating_sub(below_text.len() as u16);

            let cell = &mut canvas[(below_height, below_left)];
            cell.grapheme = Grapheme::LEFT_SEMICIRCLE;
            cell.style.fg = bg;

            for (i, g) in
                (below_left + 1..below_right.saturating_sub(1)).zip(below_text.graphemes())
            {
                let cell = &mut canvas[(below_height, i)];
                cell.grapheme = g;
                cell.style.fg = fg;
                cell.style.bg = bg;
            }

            let cell = &mut canvas[(below_height, below_right.saturating_sub(1))];
            cell.grapheme = Grapheme::RIGHT_SEMICIRCLE;
            cell.style.fg = bg;
        }
    }
}
