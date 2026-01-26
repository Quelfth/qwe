use crate::{
    custom_literal::integer::rgb,
    draw::{Rect, screen::Screen},
    editor::jump_labels::JumpLabels,
    grapheme::GraphemeExt,
};

impl JumpLabels {
    pub(super) fn draw(&self, screen: &mut Screen, rect: Rect<u16>, scroll: usize) {
        for (pos, label) in self.labels() {
            if pos.line < scroll || pos.line > scroll + rect.height() as usize {
                continue;
            }
            for (i, g) in (0..).zip(label.graphemes()) {
                let c = pos.column as u16 + rect.cols.start + i;
                if c >= rect.width() {
                    break;
                }
                let cell = &mut screen[((pos.line - scroll) as u16 + rect.rows.start, c)];
                cell.grapheme = g;
                cell.style = crate::style::FlatStyle {
                    fg: rgb! {0xffffff},
                    bg: cell.style.bg,
                    bold: true,
                    ..Default::default()
                }
            }
        }
    }
}
