use std::{collections::BTreeMap, ffi::{OsStr, OsString}, fs, iter::Sum, path::{Path, PathBuf}};

use crate::{document::diagnostics::Severity, editor::documents::{DocKey, DocOrInfo, Documents}};

pub struct Directory {
    entries: BTreeMap<OsString, Entry>,
}

pub enum Entry {
    Directory(Directory),
    File{
        name: OsString,
        doc: FileDocument,
    },
    #[expect(unused)]
    Link(PathBuf),
}

pub enum FileDocument {
    Text(DocKey),
    Binary,
    OnDisk,
}

impl Directory {
    pub fn collect(path: &Path, docs: &Documents) -> Self {
        let mut results = BTreeMap::new();

        for entry in fs::read_dir(path).into_iter().flatten() {
            let Ok(entry) = entry else {continue};
            let Ok(r#type) = entry.file_type() else {continue};
            if r#type.is_dir() {
                results.insert(entry.file_name(), Entry::Directory(Self::collect(&entry.path(), docs)));
            } else if r#type.is_file() {
                results.insert(entry.file_name(), Entry::File{
                    name: entry.path().file_name().map(|n| n.to_owned()).unwrap_or_default(),
                    doc: if let Some(key) = docs.key_from_path(&entry.path()) {
                        FileDocument::Text(key)
                    } else {
                        FileDocument::OnDisk
                    },
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

    pub fn get_mut(&mut self, dir: &OsStr) -> Option<&mut Entry> {
        self.entries.get_mut(dir)
    }
}

#[derive(Copy, Clone, Default)]
pub struct DiagnosticStatus {
    pub warnings: usize,
    pub errors: usize,
}

impl Sum for DiagnosticStatus {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), |Self{warnings: w0, errors: e0}, Self{warnings: w1, errors: e1}| Self{warnings: w0 + w1, errors: e0 + e1})
    }
}

impl Entry {
    pub fn diagnostic_status(&self, docs: &Documents) -> DiagnosticStatus {
        try {
            match self {
                Entry::Directory(directory) => directory.entries().values().map(|e| e.diagnostic_status(docs)).sum(),
                Entry::File { doc: FileDocument::Text(key), .. } => {
                    let doc = docs.by_key_or_info(*key)?;
                    let mut warnings = 0;
                    let mut errors = 0;
                    let mut add_severity = |severity: Severity| {
                        match severity {
                            Severity::Warn => warnings += 1,
                            Severity::Err => errors += 1,
                            _ => ()
                        }
                    };
                    match doc {
                        DocOrInfo::Doc(document) => {
                            for diagnostic in document.diagnostics.values() {
                                add_severity(diagnostic.severity)
                            }
                        },
                        DocOrInfo::Info(document_info) => {
                            for (_, diagnostic) in &document_info.diagnostics {
                                add_severity(diagnostic.severity)
                            }
                        },
                    }
                    DiagnosticStatus { warnings, errors }
                },
                _ => None?,
            }
        }.unwrap_or_default()
    }
}
