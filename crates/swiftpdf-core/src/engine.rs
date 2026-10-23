//! PDFium engine: loads the PDFium library and runs every PDFium call on one owner thread.
//!
//! Threading approach (PDFium is not thread-safe):
//! - PDFium keeps global state, so all calls into it must be serialized on a single thread.
//! - `PdfEngine::start` spawns one "pdfium-owner" thread that binds the library, owns the
//!   `Pdfium` instance and every open `PdfDocument`, and never lets them leave that thread.
//! - `PdfEngine` is a cheap, cloneable, `Send + Sync` handle. Each method sends a job over a
//!   channel and waits for the reply, so any thread (Tauri commands, the future RenderScheduler)
//!   can use it without locks around PDFium.
//! - We do not use pdfium-render's `thread_safe` feature: its global mutex is redundant here.
//! - PDFium is bound once per process (pdfium-render stores the bindings in a global), so only
//!   one engine can be started per process; a second `start` returns `EngineAlreadyStarted`.
//! - The owner thread gets a large stack and each job runs inside `catch_unwind`, so a hostile
//!   file can neither overflow the stack nor kill the thread through a Rust panic.
//! - Every request waits at most `request_timeout`; a stuck PDFium call yields `EngineTimeout`
//!   instead of hanging the caller. Cancellation and per-page timeouts belong to the
//!   RenderScheduler (T-021).

use std::collections::HashMap;
use std::fs::File;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use pdfium_render::prelude::{
    PdfDocument, PdfRenderConfig, Pdfium, PdfiumError, PdfiumInternalError,
};

use crate::error::CoreError;

/// Stack size of the PDFium owner thread; PDFium recurses while parsing nested objects.
pub const OWNER_THREAD_STACK_BYTES: usize = 16 * 1024 * 1024;

/// Default time a caller waits for the owner thread before giving up.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Largest bitmap edge we render, in pixels; bounds memory for hostile page sizes.
pub const MAX_RENDER_EDGE_PX: u32 = 16_384;

/// True while an engine owns the process-wide PDFium bindings.
static ENGINE_STARTED: AtomicBool = AtomicBool::new(false);

/// Identifies an open document inside one engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DocumentId(pub u64);

/// What the caller learns when a document opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentInfo {
    pub id: DocumentId,
    pub page_count: u32,
}

/// A rendered page as tightly packed RGBA8 pixels (`width * height * 4` bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedPage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// State that lives only on the PDFium owner thread.
struct OwnerState {
    pdfium: &'static Pdfium,
    documents: HashMap<DocumentId, OpenDocument>,
    next_id: u64,
}

struct OpenDocument {
    document: PdfDocument<'static>,
    page_count: u32,
}

type Job = Box<dyn FnOnce(&mut OwnerState) + Send>;

/// Handle to the PDFium owner thread. Clone it freely; all clones talk to the same thread.
#[derive(Clone)]
pub struct PdfEngine {
    jobs: mpsc::Sender<Job>,
    request_timeout: Duration,
}

impl PdfEngine {
    /// Loads PDFium from `library_path` on a new owner thread and returns a handle to it.
    pub fn start(library_path: &Path) -> Result<PdfEngine, CoreError> {
        if !library_path.is_file() {
            return Err(CoreError::PdfiumMissing {
                searched: vec![library_path.to_path_buf()],
            });
        }
        let claimed =
            ENGINE_STARTED.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);
        if claimed.is_err() {
            return Err(CoreError::EngineAlreadyStarted);
        }
        let result = spawn_owner_thread(library_path.to_path_buf());
        if result.is_err() {
            // Binding failed before PDFium was initialized, so a later attempt may still succeed.
            ENGINE_STARTED.store(false, Ordering::SeqCst);
        }
        result
    }

    /// Returns a handle that waits at most `timeout` for each request.
    pub fn with_request_timeout(&self, timeout: Duration) -> PdfEngine {
        PdfEngine {
            jobs: self.jobs.clone(),
            request_timeout: timeout,
        }
    }

    /// Opens a PDF lazily (PDFium reads only what it needs) and returns its id and page count.
    pub fn open_document(&self, path: &Path) -> Result<DocumentInfo, CoreError> {
        let path = path.to_path_buf();
        self.run(move |state| open_on_owner(state, &path))
    }

    /// Closes a document and frees its PDFium resources.
    pub fn close_document(&self, id: DocumentId) -> Result<(), CoreError> {
        self.run(move |state| match state.documents.remove(&id) {
            Some(_) => Ok(()),
            None => Err(CoreError::UnknownDocument(id.0)),
        })
    }

    /// Renders one page (0-based) at `target_width` pixels wide, keeping the aspect ratio.
    pub fn render_page(
        &self,
        id: DocumentId,
        page_index: u32,
        target_width: u32,
    ) -> Result<RenderedPage, CoreError> {
        if target_width == 0 || target_width > MAX_RENDER_EDGE_PX {
            return Err(CoreError::InvalidRenderSize(format!(
                "width {target_width} is outside 1..={MAX_RENDER_EDGE_PX}"
            )));
        }
        self.run(move |state| render_on_owner(state, id, page_index, target_width))
    }

    /// Sends `job` to the owner thread and waits for its answer.
    fn run<T, F>(&self, job: F) -> Result<T, CoreError>
    where
        T: Send + 'static,
        F: FnOnce(&mut OwnerState) -> Result<T, CoreError> + Send + 'static,
    {
        let (reply_sender, reply_receiver) = mpsc::channel();
        let wrapped: Job = Box::new(move |state: &mut OwnerState| {
            let outcome = catch_unwind(AssertUnwindSafe(|| job(state)));
            let reply = match outcome {
                Ok(result) => result,
                Err(payload) => Err(CoreError::EnginePanicked(panic_message(payload.as_ref()))),
            };
            // The caller may have timed out and gone; that is not an error here.
            let _ = reply_sender.send(reply);
        });
        if self.jobs.send(wrapped).is_err() {
            return Err(CoreError::EngineStopped);
        }
        match reply_receiver.recv_timeout(self.request_timeout) {
            Ok(reply) => reply,
            Err(mpsc::RecvTimeoutError::Timeout) => Err(CoreError::EngineTimeout {
                timeout_ms: self.request_timeout.as_millis() as u64,
            }),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(CoreError::EngineStopped),
        }
    }
}

/// Spawns the owner thread, binds PDFium on it and waits until binding has finished.
fn spawn_owner_thread(library_path: PathBuf) -> Result<PdfEngine, CoreError> {
    let (jobs_sender, jobs_receiver) = mpsc::channel::<Job>();
    let (ready_sender, ready_receiver) = mpsc::channel::<Result<(), CoreError>>();

    let spawned = thread::Builder::new()
        .name("pdfium-owner".to_string())
        .stack_size(OWNER_THREAD_STACK_BYTES)
        .spawn(move || {
            let pdfium = match bind_pdfium(&library_path) {
                Ok(pdfium) => pdfium,
                Err(error) => {
                    let _ = ready_sender.send(Err(error));
                    return;
                }
            };
            let _ = ready_sender.send(Ok(()));
            let mut state = OwnerState {
                pdfium,
                documents: HashMap::new(),
                next_id: 1,
            };
            // Runs until every engine handle is dropped.
            for job in jobs_receiver {
                job(&mut state);
            }
        });
    if let Err(error) = spawned {
        return Err(CoreError::Pdfium(format!(
            "cannot start the PDFium thread: {error}"
        )));
    }

    match ready_receiver.recv() {
        Ok(Ok(())) => Ok(PdfEngine {
            jobs: jobs_sender,
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
        }),
        Ok(Err(error)) => Err(error),
        Err(_) => Err(CoreError::EngineStopped),
    }
}

/// Loads the library and initializes PDFium; the instance lives for the rest of the process.
fn bind_pdfium(library_path: &Path) -> Result<&'static Pdfium, CoreError> {
    let bindings =
        Pdfium::bind_to_library(library_path).map_err(|error| CoreError::PdfiumLoadFailed {
            path: library_path.to_path_buf(),
            reason: error.to_string(),
        })?;
    // Leaked on purpose: documents borrow it for 'static, and PDFium cannot be re-bound anyway.
    let pdfium: &'static Pdfium = Box::leak(Box::new(Pdfium::new(bindings)));
    Ok(pdfium)
}

fn open_on_owner(state: &mut OwnerState, path: &Path) -> Result<DocumentInfo, CoreError> {
    let file = File::open(path).map_err(|error| CoreError::FileUnreadable {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;
    // A reader (not a path) keeps loading lazy and handles non-ASCII Windows paths.
    let document = state
        .pdfium
        .load_pdf_from_reader(file, None)
        .map_err(map_pdfium_error)?;

    let raw_page_count = document.pages().len();
    let page_count = u32::try_from(raw_page_count).unwrap_or(0);
    if page_count == 0 {
        return Err(CoreError::NoPages);
    }

    let id = DocumentId(state.next_id);
    state.next_id += 1;
    state.documents.insert(
        id,
        OpenDocument {
            document,
            page_count,
        },
    );
    Ok(DocumentInfo { id, page_count })
}

fn render_on_owner(
    state: &mut OwnerState,
    id: DocumentId,
    page_index: u32,
    target_width: u32,
) -> Result<RenderedPage, CoreError> {
    let Some(open) = state.documents.get(&id) else {
        return Err(CoreError::UnknownDocument(id.0));
    };
    if page_index >= open.page_count {
        return Err(CoreError::PageOutOfRange {
            index: page_index,
            page_count: open.page_count,
        });
    }
    let pdfium_index = i32::try_from(page_index).map_err(|_| CoreError::PageOutOfRange {
        index: page_index,
        page_count: open.page_count,
    })?;

    let page = open
        .document
        .pages()
        .get(pdfium_index)
        .map_err(|_| CoreError::PageUnreadable { index: page_index })?;
    // Both edges are bounded, so a page with an extreme aspect ratio cannot allocate a huge bitmap.
    let config = PdfRenderConfig::new()
        .set_target_width(target_width as i32)
        .set_maximum_height(MAX_RENDER_EDGE_PX as i32);
    let bitmap = page.render_with_config(&config).map_err(map_pdfium_error)?;

    let width = u32::try_from(bitmap.width()).unwrap_or(0);
    let height = u32::try_from(bitmap.height()).unwrap_or(0);
    if width == 0 || height == 0 {
        return Err(CoreError::InvalidRenderSize(format!(
            "PDFium produced an empty {width}x{height} bitmap"
        )));
    }
    let rgba = bitmap.as_rgba_bytes();
    let expected_len = width as usize * height as usize * 4;
    if rgba.len() != expected_len {
        return Err(CoreError::Pdfium(format!(
            "bitmap has {} bytes, expected {expected_len}",
            rgba.len()
        )));
    }
    Ok(RenderedPage {
        width,
        height,
        rgba,
    })
}

/// Turns a pdfium-render error into a typed core error.
fn map_pdfium_error(error: PdfiumError) -> CoreError {
    match error {
        PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::FormatError) => {
            CoreError::InvalidPdf
        }
        PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::PasswordError) => {
            CoreError::PasswordRequired
        }
        other => CoreError::Pdfium(other.to_string()),
    }
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(text) = payload.downcast_ref::<&str>() {
        return (*text).to_string();
    }
    if let Some(text) = payload.downcast_ref::<String>() {
        return text.clone();
    }
    "unknown panic".to_string()
}

/// File name of the PDFium library on this platform (`pdfium.dll` on Windows).
pub fn pdfium_library_file_name() -> String {
    format!(
        "{}pdfium{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    )
}

/// Places the bundled PDFium library may live, in search order.
pub fn bundled_library_candidates() -> Vec<PathBuf> {
    let file_name = pdfium_library_file_name();
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            candidates.push(exe_dir.join(&file_name));
            candidates.push(exe_dir.join("pdfium").join(&file_name));
        }
    }
    // Development builds and `cargo test` use the copy fetched by scripts/fetch-pdfium.ps1.
    if cfg!(debug_assertions) {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
        candidates.push(repo_root.join("src-tauri").join("pdfium").join(&file_name));
    }
    candidates
}

/// Returns the first existing path among `candidates`, or `PdfiumMissing` listing them all.
pub fn find_library(candidates: &[PathBuf]) -> Result<PathBuf, CoreError> {
    for candidate in candidates {
        if candidate.is_file() {
            return Ok(candidate.clone());
        }
    }
    Err(CoreError::PdfiumMissing {
        searched: candidates.to_vec(),
    })
}

/// Finds the bundled PDFium library next to the executable (or the dev copy in debug builds).
pub fn locate_bundled_library() -> Result<PathBuf, CoreError> {
    find_library(&bundled_library_candidates())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_file_name_ends_with_platform_suffix() {
        let name = pdfium_library_file_name();
        assert!(name.contains("pdfium"));
        assert!(name.ends_with(std::env::consts::DLL_SUFFIX));
    }

    #[test]
    fn find_library_reports_every_searched_path_when_missing() {
        let candidates = vec![
            PathBuf::from("no/such/dir/a.dll"),
            PathBuf::from("no/such/dir/b.dll"),
        ];
        let error = find_library(&candidates).unwrap_err();
        assert_eq!(
            error,
            CoreError::PdfiumMissing {
                searched: candidates
            }
        );
        assert!(error.to_string().contains("b.dll"));
    }

    #[test]
    fn start_with_missing_library_returns_error_without_claiming_the_engine() {
        let missing = Path::new("no/such/dir/pdfium.dll");
        let result = PdfEngine::start(missing);
        assert!(matches!(result, Err(CoreError::PdfiumMissing { .. })));
    }

    #[test]
    fn panic_message_reads_str_and_string_payloads() {
        assert_eq!(panic_message(&"boom"), "boom");
        assert_eq!(panic_message(&String::from("bang")), "bang");
        assert_eq!(panic_message(&42_u8), "unknown panic");
    }

    #[test]
    fn format_error_maps_to_invalid_pdf() {
        let error = PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::FormatError);
        assert_eq!(map_pdfium_error(error), CoreError::InvalidPdf);
    }
}
