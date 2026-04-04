use std::{ops::{Index, IndexMut}, path::Path, sync::Arc};


#[derive(Debug)]
pub struct Timeline<E> {
    pub history: TimeStack<E>,
    pub future: TimeStack<E>,
}
impl<E> Default for Timeline<E> { fn default() -> Self { Self { history: Default::default(), future: Default::default() } } }

#[derive(Copy, Clone)]
pub enum TimeDirection {
    History,
    Future,
}
impl TimeDirection { 
    pub fn rev(self) -> Self {
        use TimeDirection::*;
        match self {
            History => Future,
            Future => History,
        }
    }
}

impl<E> Index<TimeDirection> for Timeline<E> {
    type Output = TimeStack<E>;

    fn index(&self, index: TimeDirection) -> &Self::Output {
        match index {
            TimeDirection::History => &self.history,
            TimeDirection::Future => &self.future,
        }
    }
}
impl<E> IndexMut<TimeDirection> for Timeline<E> {
    fn index_mut(&mut self, index: TimeDirection) -> &mut Self::Output {
        match index {
            TimeDirection::History => &mut self.history,
            TimeDirection::Future => &mut self.future,
        }
    }
}

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

        pub fn pop(&mut self, cp: GlobalCheckpoint) -> Vec<Arc<Path>> {
            let GlobalCheckpoint(index) = cp;

            self
                .events
                .drain(index..)
                .rev()
                .filter_map(|e| match e { 
                    GlobalEvent::DocChange(p) => Some(p), 
                    _ => None 
                })
                .collect()
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
        GlobalJump(GlobalCheckpoint),
        GlobalCheckpoint,
    }

    pub enum TimeStackPop {
        Local(Vec<Change>),
        Global(GlobalCheckpoint),
        Empty,
    }

    impl TimeStack<DocumentEvent> {
        pub fn push(&mut self, change: Change) {
            self.events.push(Change(change));
        }

        pub fn checkpoint(&mut self) {
            self.events.push(Checkpoint)
        }

        pub fn pop(&mut self) -> TimeStackPop {
            while self
                .events
                .last()
                .is_some_and(|l| matches!(l, Checkpoint))
            {
                self.events.pop();
            }
            let Some(event) = self.events.pop() else { return TimeStackPop::Empty };
            match event {
                Change(change) => TimeStackPop::Local(popper(change, &mut self.events).collect()),
                GlobalJump(cp) => TimeStackPop::Global(cp),
                _ => TimeStackPop::Empty
            }
        }

        pub fn pop_global(&mut self, count: u32) -> Vec<Change> {
            let mut changes = Vec::new();
            let mut count = count;

            while let Some(change) = self.events.pop() {
                match change {
                    Change(change) => changes.push(change),
                    DocumentEvent::GlobalCheckpoint => if count > 0 {
                            count -= 1
                        } else {
                            break
                        },
                    _ => (),
                }
            }

            changes
        }

        pub fn global_checkpoint(&mut self) { self.events.push(DocumentEvent::GlobalCheckpoint) }

        pub fn push_global_jump(&mut self, checkpoint: GlobalCheckpoint) { self.events.push(GlobalJump(checkpoint)) }
    }

    fn popper(first: Change, vec: &mut Vec<DocumentEvent>) -> impl Iterator<Item = Change> + use<'_> {
        gen {
            yield first;
            while let Some(Change(change)) = vec.pop() {
                yield change;
            }
        }
    }
}
