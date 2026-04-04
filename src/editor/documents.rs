use {
    crate::document::Document, bimap::BiMap, slotmap::SlotMap, std::{mem, path::Path, sync::Arc}
};

slotmap::new_key_type! {
    pub struct DocKey;
}

#[derive(Default)]
pub struct Documents {
    docs: SlotMap<DocKey, Document>,
    paths: BiMap<Arc<Path>, DocKey>,
    save_list: Vec<DocKey>,
}

impl Documents {
    pub fn insert_pathed(&mut self, path: Arc<Path>, doc: Document) -> DocKey {
        let key = self.docs.insert(doc);
        self.paths.insert(path, key);
        key
    }

    pub fn extract_by_path(&mut self, path: &Path) -> Option<Document> {
        let (_, key) = self.paths.remove_by_left(path)?;
        self.docs.remove(key)
    }

    #[expect(unused)]
    pub fn by_path(&self, path: &Path) -> Option<&Document> {
        let key = *self.paths.get_by_left(path)?;
        self.docs.get(key)
    }

    pub fn by_path_mut(&mut self, path: &Path) -> Option<&mut Document> {
        let key = *self.paths.get_by_left(path)?;
        self.docs.get_mut(key)
    }

    pub fn by_key(&self, key: DocKey) -> Option<&Document> {
        self.docs.get(key)
    }

    pub fn by_key_mut(&mut self, key: DocKey) -> Option<&mut Document> {
        self.docs.get_mut(key)
    }

    pub fn key_from_path(&self, path: &Path) -> Option<DocKey> {
        Some(*self.paths.get_by_left(path)?)
    }

    pub fn path_from_key(&self, key: DocKey) -> Option<Arc<Path>> {
        Some(self.paths.get_by_right(&key)?.clone())
    }

    pub fn push_save(&mut self, key: DocKey) {
        self.save_list.push(key);
    }

    pub fn take_save_list(&mut self) -> Vec<DocKey> {
        mem::take(&mut self.save_list)
    }


}
