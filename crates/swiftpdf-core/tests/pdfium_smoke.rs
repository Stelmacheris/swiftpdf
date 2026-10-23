//! T-009 smoke test: real PDFium against the committed corpus.
//! Needs src-tauri/pdfium/pdfium.dll (run scripts/fetch-pdfium.ps1); fails clearly without it.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use swiftpdf_core::engine::locate_bundled_library;
use swiftpdf_core::{CoreError, PdfEngine, RenderedPage};

/// Upper bound for any single request; a hang on a hostile file fails the test instead of blocking it.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// Non-white pixels needed before a render counts as not blank.
const MIN_INK_PIXELS: usize = 200;

/// Width used for the page-1 render.
const RENDER_WIDTH: u32 = 400;

/// One engine per test process, because PDFium can only be bound once.
fn engine() -> PdfEngine {
    static ENGINE: OnceLock<PdfEngine> = OnceLock::new();
    let engine = ENGINE.get_or_init(|| {
        let library = match locate_bundled_library() {
            Ok(path) => path,
            Err(error) => panic!("{error}. Run scripts/fetch-pdfium.ps1 first."),
        };
        match PdfEngine::start(&library) {
            Ok(engine) => engine,
            Err(error) => panic!("cannot start PDFium from {}: {error}", library.display()),
        }
    });
    engine.with_request_timeout(REQUEST_TIMEOUT)
}

fn corpus_file(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/corpus/small")
        .join(name)
}

/// Every `malformed_*.pdf` in the corpus, so new hostile files are covered automatically.
fn malformed_file_names() -> Vec<String> {
    let directory = corpus_file("");
    let entries = std::fs::read_dir(&directory).expect("corpus directory is readable");
    let mut names = Vec::new();
    for entry in entries {
        let name = entry
            .expect("directory entry")
            .file_name()
            .to_string_lossy()
            .into_owned();
        if name.starts_with("malformed_") && name.ends_with(".pdf") {
            names.push(name);
        }
    }
    names.sort();
    names
}

/// True when enough pixels differ from white to rule out a blank page (one short text line is enough).
fn has_visible_content(page: &RenderedPage) -> bool {
    let mut non_white = 0_usize;
    for pixel in page.rgba.chunks_exact(4) {
        let is_white = pixel[0] > 250 && pixel[1] > 250 && pixel[2] > 250;
        if !is_white {
            non_white += 1;
        }
    }
    non_white >= MIN_INK_PIXELS
}

fn assert_opens_and_renders(file_name: &str, expected_pages: u32) {
    let engine = engine();
    let info = engine
        .open_document(&corpus_file(file_name))
        .unwrap_or_else(|error| panic!("{file_name} should open: {error}"));
    assert_eq!(info.page_count, expected_pages, "page count of {file_name}");

    let page = engine
        .render_page(info.id, 0, RENDER_WIDTH)
        .unwrap_or_else(|error| panic!("{file_name} page 1 should render: {error}"));
    assert_eq!(page.width, RENDER_WIDTH);
    assert!(page.height > 0);
    assert_eq!(page.rgba.len(), (page.width * page.height * 4) as usize);
    assert!(
        has_visible_content(&page),
        "page 1 of {file_name} rendered blank"
    );

    engine.close_document(info.id).expect("close succeeds");
}

#[test]
fn one_page_file_opens_and_renders_page_one() {
    assert_opens_and_renders("one_page_text.pdf", 1);
}

#[test]
fn hundred_page_file_opens_and_renders_page_one() {
    assert_opens_and_renders("hundred_pages_mixed.pdf", 100);
}

/// Malformed files whose damage PDFium repairs or safely ignores, with whether page 1 has content.
/// Every other `malformed_*` file must be rejected with a typed error.
const RECOVERED_MALFORMED_FILES: [(&str, bool); 4] = [
    // PDFium skips the self-referencing /Kids entry and keeps the one real page (which is empty).
    ("malformed_cyclic_page_tree.pdf", false),
    // The 100,000-deep array is never needed to draw the page.
    ("malformed_deep_arrays.pdf", true),
    // A bad startxref or shifted xref offsets trigger PDFium's xref rebuild.
    ("malformed_startxref.pdf", true),
    ("malformed_xref_offsets.pdf", true),
];

/// Returns `Some(has_content)` if PDFium is expected to recover `name`, `None` if it must be rejected.
fn expected_recovery(name: &str) -> Option<bool> {
    for (recovered_name, has_content) in RECOVERED_MALFORMED_FILES {
        if recovered_name == name {
            return Some(has_content);
        }
    }
    None
}

#[test]
fn malformed_files_never_panic_or_hang() {
    let engine = engine();
    let names = malformed_file_names();
    assert!(
        names.len() >= 10,
        "expected the malformed corpus, found {names:?}"
    );
    let mut report = Vec::new();
    for name in &names {
        let started = Instant::now();
        let outcome = open_and_render_first_page(&engine, &corpus_file(name));
        report.push(format!("{name}: {outcome:?} in {:?}", started.elapsed()));
        assert!(
            !matches!(
                outcome,
                Err(CoreError::EngineTimeout { .. })
                    | Err(CoreError::EngineStopped)
                    | Err(CoreError::EnginePanicked(_))
            ),
            "{name} hung, panicked or killed the PDFium thread: {outcome:?}"
        );
        match expected_recovery(name) {
            Some(has_content) => assert_eq!(
                outcome,
                Ok(has_content),
                "{name} should be repaired (page 1 content: {has_content})"
            ),
            None => assert!(
                outcome.is_err(),
                "{name} should be rejected, got {outcome:?}"
            ),
        }
    }
    println!("{}", report.join("\n"));

    // The engine must still work after all the hostile input.
    assert_opens_and_renders("one_page_text.pdf", 1);
}

/// Opens, renders page 1 and closes; returns whether page 1 has visible content, or the first error.
fn open_and_render_first_page(engine: &PdfEngine, path: &Path) -> Result<bool, CoreError> {
    let info = engine.open_document(path)?;
    let rendered = engine.render_page(info.id, 0, RENDER_WIDTH);
    engine.close_document(info.id)?;
    let page = rendered?;
    Ok(has_visible_content(&page))
}

#[test]
fn bad_requests_return_typed_errors() {
    let engine = engine();
    let info = engine
        .open_document(&corpus_file("one_page_text.pdf"))
        .expect("opens");

    let out_of_range = engine.render_page(info.id, 1, RENDER_WIDTH);
    assert_eq!(
        out_of_range,
        Err(CoreError::PageOutOfRange {
            index: 1,
            page_count: 1
        })
    );
    assert!(matches!(
        engine.render_page(info.id, 0, 0),
        Err(CoreError::InvalidRenderSize(_))
    ));
    engine.close_document(info.id).expect("close succeeds");

    assert_eq!(
        engine.render_page(info.id, 0, RENDER_WIDTH),
        Err(CoreError::UnknownDocument(info.id.0))
    );
    assert_eq!(
        engine.close_document(info.id),
        Err(CoreError::UnknownDocument(info.id.0))
    );
    assert!(matches!(
        engine.open_document(&corpus_file("does_not_exist.pdf")),
        Err(CoreError::FileUnreadable { .. })
    ));
}

#[test]
fn second_engine_in_the_same_process_is_refused() {
    let _ = engine();
    let library = locate_bundled_library().expect("library present");
    assert_eq!(
        PdfEngine::start(&library).err(),
        Some(CoreError::EngineAlreadyStarted)
    );
}

#[test]
fn engine_handle_works_from_many_threads() {
    let mut workers = Vec::new();
    for _ in 0..4 {
        let engine = engine();
        workers.push(std::thread::spawn(move || {
            let info = engine.open_document(&corpus_file("hundred_pages_mixed.pdf"))?;
            let page = engine.render_page(info.id, 99, 200)?;
            engine.close_document(info.id)?;
            Ok::<u32, CoreError>(page.width)
        }));
    }
    for worker in workers {
        let width = worker.join().expect("worker thread did not panic");
        assert_eq!(width, Ok(200));
    }
}
