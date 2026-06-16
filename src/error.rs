use std::io;

use thiserror::Error;

use crate::init::InitError;

#[derive(Error, Debug)]
pub enum QweError {
    #[error("{0}")]
    Init(InitError),

    #[error("{0}")]
    Setup(io::Error),

    #[error("{0}")]
    Teardown(io::Error),

    #[error("{0}")]
    Running(io::Error),
}

impl From<InitError> for QweError {
    fn from(value: InitError) -> Self {
        Self::Init(value)
    }
}

impl From<io::Error> for QweError {
    fn from(value: io::Error) -> Self {
        Self::Running(value)
    }
}