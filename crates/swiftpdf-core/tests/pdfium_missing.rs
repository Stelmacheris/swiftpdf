//! PDFium library missing or broken: the engine must return a clear error, never panic.
//! Lives in its own test binary (own process), so no other test has bound PDFium first.

use std::path::Path;

use swiftpdf_core::engine::find_library;
use swiftpdf_core::{CoreError, PdfEngine};

#[test]
fn missing_library_returns_pdfium_missing() {
    let missing = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("no-such-dir")
        .join("pdfium.dll");
    let error = PdfEngine::start(&missing).err();
    let expected = CoreError::PdfiumMissing {
        searched: vec![missing.clone()],
    };
    assert_eq!(error, Some(expected));
}

#[test]
fn missing_library_message_names_the_searched_path() {
    let missing = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("no-such-dir")
        .join("pdfium.dll");
    let error = find_library(std::slice::from_ref(&missing)).unwrap_err();
    let message = error.to_string();
    assert!(
        message.contains("PDFium library was not found"),
        "{message}"
    );
    assert!(message.contains("no-such-dir"), "{message}");
}

#[test]
fn file_that_is_not_a_library_returns_load_failed() {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join("not-a-dll");
    std::fs::create_dir_all(&directory).expect("create temp dir");
    let fake = directory.join("pdfium.dll");
    std::fs::write(&fake, b"this is not a DLL").expect("write fake library");

    let error = PdfEngine::start(&fake).err();
    assert!(
        matches!(error, Some(CoreError::PdfiumLoadFailed { .. })),
        "expected PdfiumLoadFailed, got {error:?}"
    );
}
