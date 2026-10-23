//! Usage: corpus-gen <small|large|all> [--out DIR]
//!
//! small: writes the committed files to tests/corpus/small (default).
//! large: writes the 10 MB and 1,000-page/300 MB files to D:\dev\swiftpdf-corpus (default),
//!        or to $SWIFTPDF_CORPUS_DIR.

use corpus_gen::{large, small};
use std::path::{Path, PathBuf};

const DEFAULT_LARGE_DIR: &str = r"D:\dev\swiftpdf-corpus";

fn small_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus/small")
}

fn large_dir() -> PathBuf {
    match std::env::var("SWIFTPDF_CORPUS_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => PathBuf::from(DEFAULT_LARGE_DIR),
    }
}

fn write_small(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    for (name, bytes) in small::small_files() {
        std::fs::write(dir.join(name), &bytes)?;
        println!("{name}: {} bytes", bytes.len());
    }
    Ok(())
}

fn write_large(dir: &Path) -> std::io::Result<()> {
    for path in large::write_large_files(dir)? {
        println!(
            "{}: {} bytes",
            path.display(),
            std::fs::metadata(&path)?.len()
        );
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out_override = args
        .iter()
        .position(|arg| arg == "--out")
        .and_then(|index| args.get(index + 1))
        .map(PathBuf::from);

    match args.first().map(String::as_str) {
        Some("small") => write_small(&out_override.unwrap_or_else(small_dir)),
        Some("large") => write_large(&out_override.unwrap_or_else(large_dir)),
        Some("all") => {
            write_small(&small_dir())?;
            write_large(&large_dir())
        }
        _ => {
            eprintln!("usage: corpus-gen <small|large|all> [--out DIR]");
            std::process::exit(2);
        }
    }
}
