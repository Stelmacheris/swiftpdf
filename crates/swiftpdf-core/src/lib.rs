//! SwiftPDF core library.
//!
//! Holds everything that does not depend on the UI shell: the document manager,
//! render scheduler, tile cache and editing services (see REQUIREMENTS.md §7).
//! The Tauri app in `src-tauri/` is a thin layer over this crate.
//! All PDFium access goes through [`engine::PdfEngine`], which owns the single PDFium thread.

pub mod engine;
pub mod error;

pub use engine::{DocumentId, DocumentInfo, PdfEngine, RenderedPage};
pub use error::CoreError;

/// Product name shown in the window title and dialogs.
pub const PRODUCT_NAME: &str = "SwiftPDF";

/// Version of the core crate, taken from Cargo metadata.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_name_is_swiftpdf() {
        assert_eq!(PRODUCT_NAME, "SwiftPDF");
    }

    #[test]
    fn version_is_set() {
        assert!(!VERSION.is_empty());
    }
}
