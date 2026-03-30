use std::{collections::BTreeMap, ffi::{OsStr, OsString}, fs, path::{Path, PathBuf}};

use crate::editor::documents::DocKey;

pub struct Directory {
    entries: BTreeMap<OsString, Entry>,
}

pub enum Entry {
    Directory(Directory),
    File{
        name: OsString,
        doc: FileDocument,
    },
    Link(PathBuf),
}

pub enum FileDocument {
    Text(DocKey),
    Binary,
    OnDisk,
}

impl Directory {
    pub fn collect(path: &Path) -> Self {
        let mut results = BTreeMap::new();

        for entry in fs::read_dir(path).into_iter().flatten() {
            let Ok(entry) = entry else {continue};
            let Ok(r#type) = entry.file_type() else {continue};
            if r#type.is_dir() {
                results.insert(entry.file_name(), Entry::Directory(Self::collect(&entry.path())));
            } else if r#type.is_file() {
                results.insert(entry.file_name(), Entry::File{
                    name: entry.path().file_name().map(|n| n.to_owned()).unwrap_or_default(),
                    doc: FileDocument::OnDisk,
                });
            } else if r#type.is_symlink() {
                let Ok(link) = fs::read_link(entry.path()) else {continue};
                results.insert(entry.file_name(), Entry::Link(link));
            }
        }

        Self { entries: results }
    }

    pub fn entries(&self) -> &BTreeMap<OsString, Entry> {
        &self.entries
    }

    pub fn display_entries(&self) -> impl Iterator<Item = (&OsStr, String)> {
        self.entries.iter().map(|(n, e)| {
            let name = n.to_string_lossy();

            (&**n, if matches!(e, Entry::Directory(_)) {
                format!("{name}{}", std::path::MAIN_SEPARATOR)
            } else {
                name.into()
            })
        })
    }

    pub fn get(&self, dir: &OsStr) -> Option<&Entry> {
        self.entries.get(dir)
    }
}