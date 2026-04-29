use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::{document::Document, draw::screen::Canvas, editor::{Editor, gadget::Gadget}, ix::Ix};

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
    fn on_key(&mut self, event: KeyEvent) -> Option<Box<dyn FnOnce(&mut Editor)>> {
        match event {
            KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                ..
            } => {
                if self.semantics.scroll >= self.semantics.text().line_len() {
                    self.tree.scroll += Ix::new(4);
                } else {    
                    self.semantics.scroll += Ix::new(4);
                }
                Some(Box::new(Editor::noop))
            }
            KeyEvent {
                code: KeyCode::Char('u'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                ..
            } => {
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
