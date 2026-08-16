use {
    crate::{document::{Document, diagnostics::Diagnostic}, pos::Utf16Pos},
    bimap::BiMap,
    slotmap::SlotMap,
    std::{mem, path::Path, range::Range, sync::Arc}
};

slotmap::new_key_type! {
    pub struct DocKey;
}

#[derive(Default)]
pub struct Documents {
    docs: SlotMap<DocKey, DocOrInfo>,
    paths: BiMap<Arc<Path>, DocKey>,
    save_list: Vec<Arc<Path>>,
}

impl Documents {
    pub fn pathed(&self) -> impl Iterator<Item = (Arc<Path>, &Document)> {
        self.docs.iter().filter_map(|(k, v)| Some((self.paths.get_by_right(&k)?.clone(), v.doc()?)))
    }

    pub fn pathed_mut(&mut self) -> impl Iterator<Item = (Arc<Path>, &mut Document)> {
        self.docs.iter_mut().filter_map(|(k, v)| Some((self.paths.get_by_right(&k)?.clone(), v.doc_mut()?)))
    }

    pub fn insert_pathed(&mut self, path: Arc<Path>, doc: Document) -> DocKey {
        self.insert_pathed_or_info(path, DocOrInfo::Doc(doc))
    }

    pub fn insert_pathed_or_info(&mut self, path: Arc<Path>, doc: DocOrInfo) -> DocKey {
        let key = self.docs.insert(doc);
        self.paths.insert(path, key);
        key
    }

    pub fn extract_by_path(&mut self, path: &Path) -> Option<Document> {
        let (_, key) = self.paths.remove_by_left(path)?;
        if !self.docs.get(key)?.is_doc() { return None }
        self.docs.remove(key)?.into_doc()
    }

    pub fn by_path(&self, path: &Path) -> Option<&Document> {
        let key = *self.paths.get_by_left(path)?;
        self.docs.get(key)?.doc()
    }

    pub fn by_path_mut(&mut self, path: &Path) -> Option<&mut Document> {
        self.by_path_mut_or_info(path)?.doc_mut()
    }

    pub fn by_path_mut_or_info(&mut self, path: &Path) -> Option<&mut DocOrInfo> {
        let key = *self.paths.get_by_left(path)?;
        self.docs.get_mut(key)
    }

    pub fn by_key(&self, key: DocKey) -> Option<&Document> {
        self.by_key_or_info(key)?.doc()
    }

    pub fn by_key_mut(&mut self, key: DocKey) -> Option<&mut Document> {
        self.docs.get_mut(key)?.doc_mut()
    }

    pub fn by_key_or_info(&self, key: DocKey) -> Option<&DocOrInfo> {
        self.docs.get(key)
    }

    pub fn key_from_path(&self, path: &Path) -> Option<DocKey> {
        Some(*self.paths.get_by_left(path)?)
    }

    pub fn path_from_key(&self, key: DocKey) -> Option<Arc<Path>> {
        Some(self.paths.get_by_right(&key)?.clone())
    }

    pub fn push_save(&mut self, path: Arc<Path>) {
        self.save_list.push(path);
    }

    pub fn take_save_list(&mut self) -> Vec<Arc<Path>> {
        mem::take(&mut self.save_list)
    }
}

pub enum DocOrInfo {
    Doc(Document),
    Info(DocumentInfo),
}

impl DocOrInfo {
    pub fn is_doc(&self) -> bool {
        matches!(self, DocOrInfo::Doc(_))
    }

    pub fn doc(&self) -> Option<&Document> {
        match self {
            DocOrInfo::Doc(document) => Some(document),
            DocOrInfo::Info(_) => None,
        }
    }

    pub fn doc_mut(&mut self) -> Option<&mut Document> {
        match self {
            DocOrInfo::Doc(document) => Some(document),
            DocOrInfo::Info(_) => None,
        }
    }

    pub fn into_doc(self) -> Option<Document> {
        match self {
            DocOrInfo::Doc(document) => Some(document),
            DocOrInfo::Info(_) => None,
        }
    }
}

#[derive(Default)]
pub struct DocumentInfo {
    pub diagnostics: Vec<(Range<Utf16Pos>, Diagnostic)>,
}
