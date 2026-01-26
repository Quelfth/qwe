use crate::document::Change;

#[derive(Default, Debug)]
pub struct History(Vec<HistoryElement>);

#[derive(Debug)]
pub enum HistoryElement {
    Change(Change),
    Checkpoint,
}

impl History {
    pub fn push(&mut self, change: Change) {
        self.0.push(HistoryElement::Change(change));
    }

    pub fn checkpoint(&mut self) {
        self.0.push(HistoryElement::Checkpoint)
    }

    pub fn pop(&mut self) -> impl Iterator<Item = Change> + use<'_> {
        while self
            .0
            .last()
            .is_some_and(|l| matches!(l, HistoryElement::Checkpoint))
        {
            self.0.pop();
        }
        gen {
            while let Some(HistoryElement::Change(change)) = self.0.pop() {
                yield change;
            }
        }
    }
}
