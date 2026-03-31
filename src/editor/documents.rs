use {
    crate::document::Document,
    slotmap::SlotMap,
    std::{collections::HashMap, path::Path, sync::Arc},
};

slotmap::new_key_type! {
    pub struct DocKey;
}

#[derive(Default)]
pub struct Documents {
    docs: SlotMap<DocKey, Document>,
    paths: HashMap<Arc<Path>, DocKey>,
}

impl Documents {
    pub fn insert_pathed(&mut self, path: Arc<Path>, doc: Document) -> DocKey {
        let key = self.docs.insert(doc);
        self.paths.insert(path, key);
        key
    }

    pub fn extract_by_path(&mut self, path: &Path) -> Option<Document> {
        let key = self.paths.remove(path)?;
        self.docs.remove(key)
    }

    pub fn by_path(&self, path: &Path) -> Option<&Document> {
        let key = *self.paths.get(path)?;
        self.docs.get(key)
    }

    pub fn by_path_mut(&mut self, path: &Path) -> Option<&mut Document> {
        let key = *self.paths.get(path)?;
        self.docs.get_mut(key)
    }

    pub fn by_key(&self, key: DocKey) -> Option<&Document> {
        self.docs.get(key)
    }

    pub fn key_from_path(&self, path: &Path) -> Option<DocKey> {
        Some(*self.paths.get(path)?)
    }
}
