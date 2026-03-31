use crate::{document::Document, draw::screen::Canvas, editor::gadget::Gadget};

pub struct Inspector {
    semantics: Document,
    tree: Document,
}

impl Inspector {
    pub fn new(semantics: Document, tree: Document) -> Self {
        Self { semantics, tree }
    }

    pub fn tree(&self) -> &Document {
        &self.tree
    }
}

impl Gadget for Inspector {
    fn draw(&self, mut canvas: Canvas<'_>) {
        let sem_len = self.semantics.text().line_len().inner() as u16;
        self.semantics
            .draw(canvas.take_top(sem_len));
        self.tree().draw(
            canvas.shrink_top(match sem_len {
                0 => 0,
                _ => sem_len + 1,
            }),
        )
    }
}
