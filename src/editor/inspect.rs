use crate::document::Document;

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
