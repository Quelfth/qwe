use std::ffi::OsString;

use crate::navigator::Navigator;

#[derive(Clone)]
pub enum RootPane {
    Home,
    Cwd,
    Other(OsString),
}

impl RootPane {
    pub fn text(&self) -> String {
        match self {
            RootPane::Home => format!("~{}", std::path::MAIN_SEPARATOR),
            RootPane::Cwd => format!(".{}", std::path::MAIN_SEPARATOR),
            RootPane::Other(path) => format!("{}", path.to_string_lossy()),
        }
    }
}

impl Navigator {
    pub fn root_pane(&self) -> RootPane {
        let path = &self.root_path;
        if let Some(cwd) = &self.cwd && cwd.canonicalize().ok().as_ref() == Some(&path) {
            RootPane::Cwd
        } else if let Some(home) = &self.home && home.canonicalize().ok().as_ref() == Some(&path) {
            RootPane::Home
        } else {
            RootPane::Other(path.as_os_str().to_owned())
        }
    }
}