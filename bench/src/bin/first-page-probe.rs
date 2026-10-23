//! First-page probe: measures "open + render first page" once, in a fresh process.
//!
//! Criterion repeats the operation in one process, so PDFium's font and parser caches are warm
//! after the first iteration. This probe measures what a user sees on the first open after
//! launch: bind PDFium, open the file, render page 1. `bench/run.ps1` starts it several times
//! and takes the median. It prints one JSON object on stdout.
//!
//! Usage: first-page-probe <file.pdf> [--width PX] [--pdfium PATH]

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use swiftpdf_bench::{millis, open_and_render_first_page, pdfium_library, FIRST_PAGE_WIDTH_PX};
use swiftpdf_core::PdfEngine;

struct Args {
    pdf: PathBuf,
    width: u32,
    pdfium: PathBuf,
}

fn main() -> ExitCode {
    let started = Instant::now();
    let args = match parse_args(std::env::args().skip(1).collect()) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            eprintln!("usage: first-page-probe <file.pdf> [--width PX] [--pdfium PATH]");
            return ExitCode::from(2);
        }
    };

    let bind_started = Instant::now();
    let engine = match PdfEngine::start(&args.pdfium) {
        Ok(engine) => engine,
        Err(error) => {
            eprintln!(
                "cannot start PDFium from {}: {error}",
                args.pdfium.display()
            );
            return ExitCode::FAILURE;
        }
    };
    let bind = bind_started.elapsed();

    let result = match open_and_render_first_page(&engine, &args.pdf, args.width) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("{}: {error}", args.pdf.display());
            return ExitCode::FAILURE;
        }
    };
    let total = started.elapsed();
    let memory = memory::sample();

    let file_name = args
        .pdf
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let file_bytes = std::fs::metadata(&args.pdf).map(|m| m.len()).unwrap_or(0);

    println!(
        "{{\"file\":\"{}\",\"file_bytes\":{},\"page_count\":{},\"width\":{},\"height\":{},\
         \"bind_ms\":{:.3},\"open_ms\":{:.3},\"render_ms\":{:.3},\"first_page_ms\":{:.3},\
         \"total_ms\":{:.3},\"peak_working_set_mb\":{:.1},\"working_set_mb\":{:.1},\"private_mb\":{:.1}}}",
        json_escape(&file_name),
        file_bytes,
        result.page_count,
        result.page.width,
        result.page.height,
        millis(bind),
        millis(result.timing.open),
        millis(result.timing.render),
        millis(result.timing.first_page()),
        millis(total),
        memory.peak_working_set_mb,
        memory.working_set_mb,
        memory.private_mb,
    );
    ExitCode::SUCCESS
}

fn parse_args(raw: Vec<String>) -> Result<Args, String> {
    let mut pdf = None;
    let mut width = FIRST_PAGE_WIDTH_PX;
    let mut pdfium = pdfium_library();
    let mut index = 0;
    while index < raw.len() {
        let arg = raw[index].as_str();
        if arg == "--width" || arg == "--pdfium" {
            let Some(value) = raw.get(index + 1) else {
                return Err(format!("{arg} needs a value"));
            };
            if arg == "--width" {
                width = value
                    .parse()
                    .map_err(|_| format!("--width: not a number: {value}"))?;
            } else {
                pdfium = PathBuf::from(value);
            }
            index += 2;
        } else if pdf.is_none() {
            pdf = Some(PathBuf::from(arg));
            index += 1;
        } else {
            return Err(format!("unexpected argument: {arg}"));
        }
    }
    let Some(pdf) = pdf else {
        return Err("missing the PDF path".to_string());
    };
    Ok(Args { pdf, width, pdfium })
}

fn json_escape(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

/// This process's memory counters in MB.
#[derive(Debug, Default)]
struct MemorySample {
    peak_working_set_mb: f64,
    working_set_mb: f64,
    private_mb: f64,
}

#[cfg(windows)]
mod memory {
    use super::MemorySample;
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
    };
    use windows_sys::Win32::System::Threading::GetCurrentProcess;

    const MB: f64 = 1024.0 * 1024.0;

    pub fn sample() -> MemorySample {
        let mut counters: PROCESS_MEMORY_COUNTERS_EX = unsafe { std::mem::zeroed() };
        let size = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
        counters.cb = size;
        // SAFETY: the pointer refers to a live, correctly sized PROCESS_MEMORY_COUNTERS_EX; the
        // EX struct starts with the plain struct, which is what the API signature names.
        let ok = unsafe {
            GetProcessMemoryInfo(
                GetCurrentProcess(),
                &mut counters as *mut PROCESS_MEMORY_COUNTERS_EX as *mut PROCESS_MEMORY_COUNTERS,
                size,
            )
        };
        if ok == 0 {
            return MemorySample::default();
        }
        MemorySample {
            peak_working_set_mb: counters.PeakWorkingSetSize as f64 / MB,
            working_set_mb: counters.WorkingSetSize as f64 / MB,
            private_mb: counters.PrivateUsage as f64 / MB,
        }
    }
}

#[cfg(not(windows))]
mod memory {
    use super::MemorySample;

    pub fn sample() -> MemorySample {
        MemorySample::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| item.to_string()).collect()
    }

    #[test]
    fn parses_path_width_and_library() {
        let args = parse_args(strings(&["a.pdf", "--width", "800", "--pdfium", "x.dll"])).unwrap();
        assert_eq!(args.pdf, PathBuf::from("a.pdf"));
        assert_eq!(args.width, 800);
        assert_eq!(args.pdfium, PathBuf::from("x.dll"));
    }

    #[test]
    fn width_defaults_to_first_page_width() {
        let args = parse_args(strings(&["a.pdf"])).unwrap();
        assert_eq!(args.width, FIRST_PAGE_WIDTH_PX);
    }

    #[test]
    fn rejects_missing_path_and_missing_values() {
        assert!(parse_args(Vec::new()).is_err());
        assert!(parse_args(strings(&["a.pdf", "--width"])).is_err());
        assert!(parse_args(strings(&["a.pdf", "--width", "wide"])).is_err());
        assert!(parse_args(strings(&["a.pdf", "b.pdf"])).is_err());
    }

    #[test]
    fn escapes_quotes_and_backslashes() {
        assert_eq!(json_escape(r#"a"b\c"#), r#"a\"b\\c"#);
    }
}
