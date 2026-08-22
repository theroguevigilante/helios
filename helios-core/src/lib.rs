pub mod event;

pub use event::{NetworkInfo, ProcessInfo, TraceInfo, UniversalEvent};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Format detection error: {0}")]
    DetectionError(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, Error>;
