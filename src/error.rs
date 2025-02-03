use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("Failed to read directory '{0}'")]
    DirReadError(String),
    #[error("Terminal error: {0}")]
    TerminalError(String),
    #[error("Analysis error: {0}")]
    AnalysisError(String),
}

pub type AppResult<T> = Result<T, AppError>;
