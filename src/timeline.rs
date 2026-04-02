use std::{path::Path, sync::Arc};


#[derive(Debug)]
pub struct Timeline<E> {
    pub history: TimeStack<E>,
    pub future: TimeStack<E>,
}
impl<E> Default for Timeline<E> { fn default() -> Self { Self { history: Default::default(), future: Default::default() } } }

#[derive(Debug)]
pub struct TimeStack<E> {
    events: Vec<E>,
}
impl<E> Default for TimeStack<E> { fn default() -> Self { Self { events: Default::default() } } }

pub mod global {
    use {
        super::*,
        GlobalEvent::*,
    };

    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub struct GlobalCheckpoint(usize);

    pub enum GlobalEvent {
        DocChange(Arc<Path>),
        Checkpoint,
    }

    impl TimeStack<GlobalEvent> {
        pub fn push_doc_change(&mut self, doc: Arc<Path>) {
            self.events.push(DocChange(doc));
        }

        pub fn checkpoint(&mut self) -> GlobalCheckpoint {
            self.events.push(Checkpoint);
            GlobalCheckpoint(self.events.len())
        }
    }
}

pub mod document {
    use {
        super::{*, global::GlobalCheckpoint},
        crate::document::Change,
        DocumentEvent::*,
    };

    #[derive(Debug)]
    pub enum DocumentEvent {
        Change(Change),
        Checkpoint,
        GlobalJump(global::GlobalCheckpoint),
        GlobalCheckpoint,
    }

    impl TimeStack<DocumentEvent> {
        pub fn push(&mut self, change: Change) {
            self.events.push(Change(change));
        }

        pub fn checkpoint(&mut self) {
            self.events.push(Checkpoint)
        }

        pub fn pop(&mut self) -> impl Iterator<Item = Change> + use<'_> {
            while self
                .events
                .last()
                .is_some_and(|l| matches!(l, Checkpoint))
            {
                self.events.pop();
            }
            gen {
                while let Some(Change(change)) = self.events.pop() {
                    yield change;
                }
            }
        }

        pub fn global_checkpoint(&mut self) { self.events.push(DocumentEvent::GlobalCheckpoint) }

        pub fn push_global_jump(&mut self, checkpoint: GlobalCheckpoint) { self.events.push(GlobalJump(checkpoint)) }

        //pub fn push_global_changes(&mut self, checkpoint: GlobalCheckpoint, changes: impl IntoIterator<Item = Change>) {
        //    self.global_checkpoint();
        //    self.events.extend(changes.into_iter().map(Change));
        //    self.push_global_jump(checkpoint);
        //}
    }
}