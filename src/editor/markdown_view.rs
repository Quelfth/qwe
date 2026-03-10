use crate::{
    color,
    draw::screen::Canvas,
    editor::gadget::Gadget,
    grapheme::{Grapheme, GraphemeExt},
    style::Style,
};

pub struct MarkdownView {
    scroll: usize,
    text: String,
}

impl MarkdownView {
    pub fn new(text: String) -> Self {
        Self {
            scroll: 0,
            text,
        }
    }
}

pub struct MarkdownGadget {
    view: MarkdownView,
}

impl MarkdownGadget {
    pub fn empty() -> Self {
        Self {
            view: MarkdownView::new(String::new()),
        }
    }

    pub fn new(view: String) -> Self {
        Self {
            view: MarkdownView::new(view),
        }
    }
}

impl Gadget for MarkdownGadget {

    fn draw(&self, mut canvas: Canvas<'_>) {
        let style = (Style::fg(color::MD_FG) + Style::bg(color::MD_BG)).into();
        for (i, line) in (0..canvas.height()).zip(self.view.text.lines().skip(self.view.scroll)) {
            let mut js = 0..canvas.width();
            for (j, g) in js.by_ref().zip(line.graphemes()) {
                let cell = &mut canvas[(i, j)];
                cell.grapheme = g;
                cell.style = style;
            }
            for j in js.start - 1..js.end {
                let cell = &mut canvas[(i, j)];
                cell.grapheme = Grapheme::SPACE;
                cell.style = style;
            }
        }
    }
}
