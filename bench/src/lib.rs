//! Shared helpers for the SwiftPDF benchmarks (T-010).
//!
//! The criterion benches (`benches/first_page.rs`) and the fresh-process probe
//! (`src/bin/first-page-probe.rs`) both measure the same operation: open a PDF, render its
//! first page at a screen-sized width, close it. That is the core part of the REQUIREMENTS.md
//! §6.1 budgets "10 MB PDF → first sharp page < 200 ms" and "1,000-page / 300 MB PDF → first
//! page < 400 ms". `bench/run.ps1` runs both and writes the results as JSON.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use swiftpdf_core::{CoreError, PdfEngine, RenderedPage};

/// Width of the first-page render: an A4/Letter page fit to width in a maximized 1080p window.
pub const FIRST_PAGE_WIDTH_PX: u32 = 1280;

/// The generated 10 MB file (`cargo run --release -p swiftpdf-corpus-gen -- large`).
pub const FILE_10MB: &str = "large_10mb.pdf";

/// The generated 1,000-page / 300 MB file.
pub const FILE_300MB: &str = "large_1000pages_300mb.pdf";

/// Where corpus-gen writes the large files unless `SWIFTPDF_CORPUS_DIR` says otherwise.
pub const DEFAULT_CORPUS_DIR: &str = r"D:\dev\swiftpdf-corpus";

/// Folder that holds the large generated corpus files.
pub fn corpus_dir() -> PathBuf {
    match std::env::var_os("SWIFTPDF_CORPUS_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(DEFAULT_CORPUS_DIR),
    }
}

/// Full path of one large corpus file.
pub fn corpus_file(name: &str) -> PathBuf {
    corpus_dir().join(name)
}

/// The PDFium library fetched by `scripts/fetch-pdfium.ps1`, or `SWIFTPDF_PDFIUM` if set.
pub fn pdfium_library() -> PathBuf {
    match std::env::var_os("SWIFTPDF_PDFIUM") {
        Some(path) => PathBuf::from(path),
        None => repo_pdfium_library(),
    }
}

/// `<repo>/src-tauri/pdfium/pdfium.dll`, found from this crate's manifest folder.
pub fn repo_pdfium_library() -> PathBuf {
    let file_name = swiftpdf_core::engine::pdfium_library_file_name();
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("src-tauri")
        .join("pdfium")
        .join(file_name)
}

/// Fails with a clear message when a large corpus file has not been generated yet.
pub fn require_corpus_file(name: &str) -> Result<PathBuf, String> {
    let path = corpus_file(name);
    if path.is_file() {
        return Ok(path);
    }
    Err(format!(
        "{} is missing. Generate it with `cargo run --release -p swiftpdf-corpus-gen -- large` \
         or point SWIFTPDF_CORPUS_DIR at the folder that has it.",
        path.display()
    ))
}

/// How long each step of "open + render first page" took.
#[derive(Debug, Clone, Copy)]
pub struct FirstPageTiming {
    pub open: Duration,
    pub render: Duration,
}

impl FirstPageTiming {
    /// Open plus render: the time until the first page's pixels exist.
    pub fn first_page(&self) -> Duration {
        self.open + self.render
    }
}

/// What one "open + render first page" produced.
pub struct FirstPageResult {
    pub timing: FirstPageTiming,
    pub page_count: u32,
    pub page: RenderedPage,
}

/// Opens `path`, renders page 1 at `width` pixels and closes the document again.
pub fn open_and_render_first_page(
    engine: &PdfEngine,
    path: &Path,
    width: u32,
) -> Result<FirstPageResult, CoreError> {
    let open_started = Instant::now();
    let info = engine.open_document(path)?;
    let open = open_started.elapsed();

    let render_started = Instant::now();
    let rendered = engine.render_page(info.id, 0, width);
    let render = render_started.elapsed();

    // Close even when rendering failed, so a long benchmark run does not leak documents.
    engine.close_document(info.id)?;
    let page = rendered?;

    Ok(FirstPageResult {
        timing: FirstPageTiming { open, render },
        page_count: info.page_count,
        page,
    })
}

/// Milliseconds as a float, for JSON output.
pub fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_pdfium_library_points_into_src_tauri() {
        let path = repo_pdfium_library();
        let text = path.to_string_lossy().replace('\\', "/");
        assert!(text.contains("src-tauri/pdfium/"), "{text}");
        assert!(text.ends_with(std::env::consts::DLL_SUFFIX), "{text}");
    }

    #[test]
    fn corpus_file_is_inside_corpus_dir() {
        assert_eq!(corpus_file(FILE_10MB), corpus_dir().join(FILE_10MB));
    }

    #[test]
    fn first_page_time_is_open_plus_render() {
        let timing = FirstPageTiming {
            open: Duration::from_millis(30),
            render: Duration::from_millis(70),
        };
        assert_eq!(timing.first_page(), Duration::from_millis(100));
    }

    #[test]
    fn millis_converts_durations() {
        assert_eq!(millis(Duration::from_micros(1500)), 1.5);
    }

    #[test]
    fn missing_corpus_file_names_the_generator() {
        let error = require_corpus_file("no-such-file-for-tests.pdf").unwrap_err();
        assert!(error.contains("swiftpdf-corpus-gen"), "{error}");
    }
}
