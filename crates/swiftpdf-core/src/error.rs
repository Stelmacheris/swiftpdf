//! Error type shared by the core crate.

use std::path::PathBuf;

/// Every failure the core can report. Malformed or hostile files always end up here, never in a panic.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CoreError {
    /// No PDFium library exists at any of the searched paths.
    #[error(
        "the PDFium library was not found (searched: {})",
        format_paths(searched)
    )]
    PdfiumMissing { searched: Vec<PathBuf> },

    /// The PDFium library exists but could not be loaded or bound.
    #[error("the PDFium library at {path} could not be loaded: {reason}")]
    PdfiumLoadFailed { path: PathBuf, reason: String },

    /// PDFium can only be bound once per process, and an engine already owns it.
    #[error("the PDFium engine is already running in this process")]
    EngineAlreadyStarted,

    /// The PDFium owner thread has stopped (it exited or crashed).
    #[error("the PDFium engine has stopped")]
    EngineStopped,

    /// The PDFium owner thread did not answer in time (it may be stuck on a hostile file).
    #[error("the PDFium engine did not answer within {timeout_ms} ms")]
    EngineTimeout { timeout_ms: u64 },

    /// PDFium panicked inside the binding layer while handling a request.
    #[error("internal error while handling the document: {0}")]
    EnginePanicked(String),

    /// The file could not be read from disk.
    #[error("cannot read {path}: {reason}")]
    FileUnreadable { path: PathBuf, reason: String },

    /// The file is not a PDF or is damaged beyond repair.
    #[error("the file is not a valid PDF or is damaged")]
    InvalidPdf,

    /// The file is encrypted and needs a password (handled in a later milestone).
    #[error("the document is password protected")]
    PasswordRequired,

    /// The file opened but contains no usable pages.
    #[error("the document has no pages")]
    NoPages,

    /// No open document has this id.
    #[error("no open document has id {0}")]
    UnknownDocument(u64),

    /// The page index is outside the document.
    #[error("page {index} does not exist (the document has {page_count} pages)")]
    PageOutOfRange { index: u32, page_count: u32 },

    /// The page exists in the page count but PDFium cannot load it (for example a broken page tree).
    #[error("page {index} cannot be read; the document may be damaged")]
    PageUnreadable { index: u32 },

    /// The requested bitmap size is zero or too large.
    #[error("invalid render size: {0}")]
    InvalidRenderSize(String),

    /// Any other error reported by PDFium.
    #[error("PDFium error: {0}")]
    Pdfium(String),
}

fn format_paths(paths: &[PathBuf]) -> String {
    let mut names = Vec::new();
    for path in paths {
        names.push(path.display().to_string());
    }
    names.join("; ")
}
