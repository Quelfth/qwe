use std::any::Any;

use crate::{
    custom_literal::integer::rgb, document::{Document, diagnostics::Severity}, draw::screen::Canvas, editor::cursors::Cursors, grapheme::{Grapheme, GraphemeExt}, ix::Ix, style::Style,
};

impl Document {
    pub fn draw_edge_indicators(&self, mut canvas: Canvas<'_>) {
        #[derive(Copy, Clone, Default)]
        struct Counts {
            errors: u32,
            warnings: u32,
            cursors: u32,
        }
        impl Counts {
            fn each(self) -> impl Iterator<Item = u32> {
                [self.errors, self.warnings, self.cursors].into_iter()
            }
            fn num(self) -> u32 {
                self.each().filter(|&c| c > 0).count() as _
            }

            fn boxes(self, leader: &str) -> impl Iterator<Item = (String, Style, (Grapheme, Grapheme))> {
                self.each()
                    .zip([
                        (Severity::Err.style() + Style::italic(), HEX_ENDS),
                        (Severity::Warn.style() + Style::italic(), HEX_ENDS),
                        (Style::fg(rgb!(0x20D0D0)) + Style::bg(rgb!(0x103050)), ROUND_ENDS)
                    ])
                    .filter_map(move |(c, (s, e))| {
                        (c != 0).then(|| (format!("{leader} {c}"), s, e))
                    })
            }
        }

        let mut above = Counts::default();
        let mut below = Counts::default();

        let screen_range = self.scroll..self.scroll + *self.view_height.lock();

        if let Some(cursors) = &self.cursors {
            for range in cursors.line_ranges() {
                if range.end <= screen_range.start {
                    above.cursors += 1;
                }
                if range.start >= screen_range.end {
                    below.cursors += 1;
                }
            }
        }

        for (range, d) in self.diagnostics.ranges() {
            use Severity::*;
            if self.text().line_of_byte(range.end).is_some_and(|line| line <= screen_range.start) {
                match d.severity {
                    Warn => above.warnings += 1,
                    Err => above.errors += 1,
                    _ => (),
                }
            }
            if self.text().line_of_byte(range.start).is_some_and(|line| line >= screen_range.end) {
                match d.severity {
                    Warn => below.warnings += 1,
                    Err => below.errors += 1,
                    _ => (),
                }
            }
        }

        const VERT_MARGIN: u16 = 1;
        const RIGHT_MARGIN: u16 = 4;

        const ROUND_ENDS: (Grapheme, Grapheme) = (Grapheme::LEFT_SEMICIRCLE, Grapheme::RIGHT_SEMICIRCLE);
        const HEX_ENDS: (Grapheme, Grapheme) = (Grapheme::LEFT_TRIANGLE, Grapheme::RIGHT_TRIANGLE);

        fn columns_in_boxes(boxes: &[(String, impl Any, impl Any)]) -> u16 {
            boxes
                .iter()
                .map(|(t, _, _)| t.graphemes().map(|g| g.columns().inner() as u16).sum::<u16>())
                .map(|c| c + 2)
                .sum()
        }

        let above_num = above.num() as u16;
        if above_num > 0 {
            let text = above.boxes("▲").collect::<Vec<_>>();
            let above_right = canvas.width().saturating_sub(RIGHT_MARGIN);
            let above_left = above_right.saturating_sub(columns_in_boxes(&text) + (above_num - 1));

            let mut cursor = canvas.at((VERT_MARGIN, above_left));

            for (text, style, ends) in text {
                _= cursor
                    .write_box(
                        text,
                        style,
                        ends,
                    );
                cursor.blank(Ix::new(1))
            }

        }
        let below_num = below.num() as u16;
        if below_num > 0 {
            let below_height = canvas.height().saturating_sub(VERT_MARGIN + 1);
            let text = below.boxes("▼").collect::<Vec<_>>();
            let below_right = canvas.width().saturating_sub(RIGHT_MARGIN);
            let below_left = below_right.saturating_sub(columns_in_boxes(&text) + (below_num - 1));

            let mut cursor = canvas.at((below_height, below_left));

            for (text, style, ends) in text {
                _= cursor
                    .write_box(
                        text,
                        style,
                        ends,
                    );
                cursor.blank(Ix::new(1))
            }
        }
    }
}
