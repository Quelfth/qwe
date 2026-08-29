use std::{io, path::Path, sync::Arc};

use thiserror::Error;

use crate::{Args, pathed_file::*, pos::Pos};


pub fn init(args: Args) -> Result<InitState, InitError> {
    use InitError::*;
    let Args { path, new, dirs, find, line, autosave } = args;
    let path = if let Some(path) = path {
        Some(path)
    } else {
        if let Some(find) = find {
            let dir = std::env::current_dir().map_err(NoCwd)?;
            let mut options = walkdir::WalkDir::new(dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().to_owned())
                .collect::<Vec<_>>();
            options.sort_by_key(|o| o.as_os_str().len());
            Some(options
                .into_iter()
                .find(|e| find.iter().all(|f| e.to_string_lossy().contains(f)))
                .ok_or(FailedToFind(find))?
            )
        } else {
            None
        }
    };
    let path = if let Some(path) = path {
        let path: Arc<Path> = path.into();
        Some(if !new {
            PathedFile::open(path.clone()).map_err(|e| CouldNotOpen(path, e))?
        } else if !dirs {
            PathedFile::create(path.clone()).map_err(|e| CouldNotCreate(path, e))?
        } else {
            PathedFile::create_with_dirs(path.clone()).map_err(|e| {
                use CreateWithDirsPathedFileError::*;
                match e {
                    Dirs(e) => CouldNotCreateDirs(path, e),
                    File(e) => CouldNotCreate(path, e),
                }
            })?
        })
    } else {
        None
    };
    Ok(InitState {
        doc: path,
        pos: line,
        autosave,
    })
}

pub struct InitState {
    pub doc: Option<PathedFile>,
    pub pos: Option<Pos>,
    pub autosave: bool,
}

#[derive(Error, Debug)]
pub enum InitError {
    #[error("working directory does not exist:\n{0}")]
    NoCwd(io::Error),

    #[error("could not open file {0}:\n{1}")]
    CouldNotOpen(Arc<Path>, io::Error),

    #[error("could not create file {0}:\n{1}")]
    CouldNotCreate(Arc<Path>, io::Error),
    
    #[error("could not create directory {0}:\n{1}")]
    CouldNotCreateDirs(Arc<Path>, io::Error),

    #[error("failed to find file for search term {0:?}")]
    FailedToFind(Vec<String>),
}
