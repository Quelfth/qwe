use crate::{
    color, draw::screen::Canvas, editor::gadget::Gadget, grapheme::{Grapheme, GraphemeExt}, markdown::{MdContext, MdCxCache, MdDraw}, style::{FlatStyle, Style}
};

pub struct MarkdownView {
    #[allow(unused)]
    scroll: usize,
    ast: markdown::mdast::Node,
    doc_cache: MdCxCache,
}

impl MarkdownView {
    pub fn new(text: String) -> Self {
        let ast = markdown::to_mdast(&text, &Default::default()).unwrap();
        Self {
            scroll: 0,
            ast,
            doc_cache: Default::default(),
        }
    }
}

impl MarkdownView {
    pub fn draw(&self, mut canvas: Canvas<'_>) {
        canvas.fill_bg(color::MD_BG);
        _= self.ast.draw(MdContext::new(self.doc_cache.clone()), &mut canvas.at((0, 0)));
    }
}

pub struct MarkdownGadget {
    view: MarkdownView,
}

impl MarkdownGadget {
    pub fn new(view: String) -> Self {
        Self {
            view: MarkdownView::new(view),
        }
    }
}

impl Gadget for MarkdownGadget {
    fn draw(&self, canvas: Canvas<'_>) {
        self.view.draw(canvas)
    }
}
