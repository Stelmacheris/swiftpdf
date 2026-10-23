//! Criterion benches for the §6.1 first-page budgets (T-010).
//!
//! All benches share one PDFium engine (PDFium binds once per process), so after the first
//! iteration PDFium's caches and the OS file cache are warm: these are steady-state numbers.
//! The fresh-process number comes from `first-page-probe`. Criterion writes its estimates to
//! `<target>/criterion/first_page/<bench>/new/estimates.json`; `bench/run.ps1` reads them.
//!
//! Run: `cargo bench -p swiftpdf-bench --bench first_page`

use std::path::Path;
use std::time::Duration;

use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, Criterion};
use swiftpdf_bench::{
    open_and_render_first_page, pdfium_library, require_corpus_file, FILE_10MB, FILE_300MB,
    FIRST_PAGE_WIDTH_PX,
};
use swiftpdf_core::PdfEngine;

fn start_engine() -> PdfEngine {
    let library = pdfium_library();
    match PdfEngine::start(&library) {
        Ok(engine) => engine,
        Err(error) => panic!(
            "cannot start PDFium from {}: {error}. Run scripts/fetch-pdfium.ps1 first.",
            library.display()
        ),
    }
}

fn corpus_path(name: &str) -> std::path::PathBuf {
    match require_corpus_file(name) {
        Ok(path) => path,
        Err(message) => panic!("{message}"),
    }
}

fn bench_open_and_render(
    group: &mut BenchmarkGroup<'_, WallTime>,
    engine: &PdfEngine,
    name: &str,
    path: &Path,
) {
    group.bench_function(name, |bencher| {
        bencher.iter(|| {
            open_and_render_first_page(engine, path, FIRST_PAGE_WIDTH_PX)
                .expect("open and render the first page")
        })
    });
}

fn first_page(criterion: &mut Criterion) {
    let engine = start_engine();
    let file_10mb = corpus_path(FILE_10MB);
    let file_300mb = corpus_path(FILE_300MB);

    let mut group = criterion.benchmark_group("first_page");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(5));

    bench_open_and_render(&mut group, &engine, "open_render_10mb", &file_10mb);
    bench_open_and_render(&mut group, &engine, "open_render_300mb", &file_300mb);

    // Rendering page 1 again on an open document: the core of a zoom re-render (§6.1 zoom).
    let open = engine
        .open_document(&file_10mb)
        .expect("open the 10 MB file");
    group.bench_function("rerender_page1_10mb", |bencher| {
        bencher.iter(|| {
            engine
                .render_page(open.id, 0, FIRST_PAGE_WIDTH_PX)
                .expect("render page 1")
        })
    });
    group.finish();
    engine
        .close_document(open.id)
        .expect("close the 10 MB file");
}

criterion_group!(benches, first_page);
criterion_main!(benches);
