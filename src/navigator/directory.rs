use std::{ffi::{OsStr, OsString}, fs, path::{Path, PathBuf}};

use crate::editor::documents::DocKey;

pub struct Directory {
    entries: Vec<Entry>,
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
        let mut results = Vec::new();

        for entry in fs::read_dir(path).into_iter().flatten() {
            let Ok(entry) = entry else {continue};
            let Ok(r#type) = entry.file_type() else {continue};
            if r#type.is_dir() {
                results.push(Entry::Directory(Self::collect(&entry.path())));
            } else if r#type.is_file() {
                results.push(Entry::File{
                    name: entry.path().file_name().map(|n| n.to_owned()).unwrap_or_default(),
                    doc: FileDocument::OnDisk,
                });
            } else if r#type.is_symlink() {
                let Ok(link) = fs::read_link(entry.path()) else {continue};
                results.push(Entry::Link(link));
            }
        }

        Self { entries: results }
    }
}