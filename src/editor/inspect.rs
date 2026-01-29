use crate::{document::Document, draw::screen::Canvas, editor::gadget::Gadget};

pub struct Inspector {
    tree: Document,
}

impl Inspector {
    pub fn new(tree: Document) -> Self {
        Self { tree }
    }

    pub fn tree(&self) -> &Document {
        &self.tree
    }
}

impl Gadget for Inspector {
    fn draw(&self, canvas: Canvas<'_>) {
        self.tree().draw(canvas, |_| Default::default())
    }
}
