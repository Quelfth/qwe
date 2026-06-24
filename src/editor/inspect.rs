use std::{iter, range::Range};

use crate::{document::{Document, semtoks::SemanticToken, tree::MetaTree}, draw::screen::Canvas, editor::{Editor, gadget::Gadget}, ix::{Byte, Ix}, key::{KeyOrChar, key}, lang::Language, util::{RangeOverlap as _, pretty_node}};

pub struct Inspector {
    semantics: Document,
    injections: Vec<Document>,
    tree: Document,
}

impl Inspector {
    pub fn new<'a>(semantics: impl IntoIterator<Item = (Range<Ix<Byte>>, &'a SemanticToken)>, tree: &MetaTree, range: Range<Ix<Byte>>) -> Self {
        let semantics = Document::new(
            None,
            semantics
                .into_iter()
                .filter(|(r, _)| r.overlaps(range))
                .map(|(_, s)| {
                    iter::once((*s.r#type).to_owned())
                        .chain(s.mods.iter().map(|m| " ".to_owned() + m))
                        .collect::<String>()
                        + "\n"
                })
                .collect::<String>(),
            None,
        );
        let mut injection_tree = tree;
        let tree = Document::new(
            Some(Language::Query),
            pretty_node(
                tree.tree.root_node()
                    .descendant_for_byte_range(range.start.inner(), range.end.inner())
                    .unwrap(),
            ),
            None,
        );
        let mut injections = Vec::new();
        while let Some(injection) = injection_tree.injections.iter().find(|i| i.range.overlaps(range)) {
            injections.push(Document::new(
                Some(Language::Query),
                pretty_node(
                    injection
                        .tree.tree
                        .root_node()
                        .descendant_for_byte_range(range.start.inner(), range.end.inner())
                        .unwrap(),
                ),
                None,
            ));
            injection_tree = &injection.tree;
        }
        Self { semantics, injections, tree }
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
        let mut canvas = canvas.shrink_top(match sem_len {
            0 => 0,
            _ => sem_len + 1,
        });
        let mut shift = 0;
        for injection in self.injections.iter().rev() {
            let inj_len = injection.text().line_len().saturating_sub(self.semantics.scroll).inner() as u16;
            injection.draw(canvas.shrink_top(shift).take_top(inj_len));
            shift += inj_len + 1;
        }
        self.tree().draw(canvas.shrink_top(shift))
    }
}
