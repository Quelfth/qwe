use {
    std::{
        collections::HashMap,
        path::Path,
        sync::Arc,
    },
    slotmap::{
        SlotMap,
    },
    crate::document::Document,
};

slotmap::new_key_type!{
    pub struct DocKey;
}

#[derive(Default)]
pub struct BackgroundDocuments {
    docs: SlotMap<DocKey, Document>,
    paths: HashMap<Arc<Path>, DocKey>,
}

impl BackgroundDocuments {
    pub fn insert_pathed(&mut self, path: Arc<Path>, doc: Document) {
        let key = self.docs.insert(doc);
        self.paths.insert(path, key);
    }

    pub fn extract_by_path(&mut self, path: &Path) -> Option<Document> {
        let key = self.paths.remove(path)?;
        self.docs.remove(key)
    }
}