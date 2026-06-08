use crate::{document::Document, draw::screen::Canvas, editor::{Editor, gadget::Gadget}, ix::Ix, key::{KeyOrChar, key}};

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
    fn on_key(&mut self, event: KeyOrChar) -> Option<Box<dyn FnOnce(&mut Editor)>> {
        use KeyOrChar::Key;
        match event {
            Key(key![ctrl d] | key![scroll down]) => {
                if self.semantics.scroll >= self.semantics.text().line_len() {
                    self.tree.scroll += Ix::new(4);
                } else {    
                    self.semantics.scroll += Ix::new(4);
                }
                Some(Box::new(Editor::noop))
            }
            Key(key![ctrl u] | key![scroll up]) => {
                if self.tree.scroll == Ix::new(0) {
                    self.semantics.scroll = self.semantics.scroll.saturating_sub(Ix::new(4));
                } else {
                    self.tree.scroll = self.tree.scroll.saturating_sub(Ix::new(4));
                }
                Some(Box::new(Editor::noop))
            }
            _ => None
        }
    }

    fn draw(&self, mut canvas: Canvas<'_>) {
        let sem_len = self.semantics.text().line_len().saturating_sub(self.semantics.scroll).inner() as u16;
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
