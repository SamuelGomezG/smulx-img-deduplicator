use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum DeduplicatorError {
    #[error("Cannot read image {path}: {source}")]
    ImageRead {
        path: PathBuf,
        #[source]
        source: image::ImageError,
    },

    #[error("I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Permission denied when trying to delete {path}")]
    DeletePermission { path: PathBuf },

    #[error("TUI error: {0}")]
    Tui(#[from] std::io::Error),
}
