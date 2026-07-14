use std::{fs, io, path::Path, sync::Arc};

use thiserror::Error;

pub struct PathedFile {
    pub path: Arc<Path>,
    pub file: String,
}

impl PathedFile {
    pub fn empty(path: Arc<Path>) -> Self {
        Self {
            file: "".to_owned(),
            path,
        }
    }

    pub fn open(path: Arc<Path>) -> io::Result<Self> {
        Ok(Self {
            file: fs::read_to_string(&path)?,
            path,
        })
    }

    pub fn create(path: Arc<Path>) -> io::Result<Self> {
        fs::File::create_new(&path)?;
        Ok(Self::empty(path))
    }

    pub fn create_with_dirs(path: Arc<Path>) -> Result<Self, CreateWithDirsPathedFileError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(CreateWithDirsPathedFileError::Dirs)?;
        }
        Self::create(path).map_err(CreateWithDirsPathedFileError::File)
    }
}

#[derive(Error, Debug)]
pub enum CreateWithDirsPathedFileError {
    #[error("{0}")]
    Dirs(io::Error),
    #[error("{0}")]
    File(io::Error),
}