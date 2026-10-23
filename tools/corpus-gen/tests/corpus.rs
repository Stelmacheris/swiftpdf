//! Verifies the generated corpus: valid files parse, malformed files are malformed, committed files are current.

use corpus_gen::{large, small, verify};
use std::collections::HashMap;
use std::path::PathBuf;

fn committed_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus/small")
}

fn files() -> HashMap<&'static str, Vec<u8>> {
    small::small_files().into_iter().collect()
}

fn page_count(bytes: &[u8]) -> usize {
    lopdf::Document::load_mem(bytes)
        .expect("lopdf opens the file")
        .get_pages()
        .len()
}

#[test]
fn committed_files_match_the_generator() {
    let generated = files();
    for (name, bytes) in &generated {
        let on_disk = std::fs::read(committed_dir().join(name))
            .unwrap_or_else(|_| panic!("{name} is missing: run `corpus-gen small`"));
        assert!(&on_disk == bytes, "{name} is stale: run `corpus-gen small`");
    }
    let on_disk_count = std::fs::read_dir(committed_dir())
        .unwrap()
        .filter(|entry| {
            entry
                .as_ref()
                .unwrap()
                .path()
                .extension()
                .is_some_and(|e| e == "pdf")
        })
        .count();
    assert_eq!(
        on_disk_count,
        generated.len(),
        "unexpected extra files in tests/corpus/small"
    );
}

#[test]
fn committed_corpus_is_under_five_megabytes() {
    let total: usize = files().values().map(Vec::len).sum();
    assert!(total < 5_000_000, "corpus is {total} bytes");
}

#[test]
fn readme_documents_every_file() {
    let readme = std::fs::read_to_string(committed_dir().join("../README.md")).unwrap();
    let names = files()
        .into_keys()
        .chain([large::TEN_MB_NAME, large::THOUSAND_PAGE_NAME]);
    for name in names {
        assert!(readme.contains(name), "README.md does not mention {name}");
    }
}

#[test]
fn valid_files_pass_structure_check_and_open_in_lopdf() {
    let expected_pages = [
        ("one_page_text.pdf", 1),
        ("hundred_pages_mixed.pdf", 100),
        ("outline_and_links.pdf", 6),
        ("acroform.pdf", 1),
        ("xfa_form.pdf", 1),
        ("attachments.pdf", 1),
        ("rotated_pages.pdf", 4),
        ("mixed_page_sizes.pdf", 6),
    ];
    let generated = files();
    for (name, pages) in expected_pages {
        let bytes = &generated[name];
        verify::check_structure(bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(page_count(bytes), pages, "{name}");
    }
}

#[test]
fn valid_files_contain_the_requested_features() {
    let generated = files();
    let text = |name: &str| String::from_utf8_lossy(&generated[name]).into_owned();
    for name in ["acroform.pdf", "xfa_form.pdf"] {
        for field in ["/FT /Tx", "/FT /Btn", "/Ff 32768", "/Ff 131072", "/FT /Ch"] {
            assert!(text(name).contains(field), "{name} lacks {field}");
        }
    }
    assert!(text("xfa_form.pdf").contains("/XFA"));
    assert!(!text("acroform.pdf").contains("/XFA"));
    assert!(text("attachments.pdf").contains("/EmbeddedFiles"));
    assert!(text("outline_and_links.pdf").contains("/Outlines"));
    assert!(text("outline_and_links.pdf").contains("/S /URI"));
    for angle in [90, 180, 270] {
        assert!(text("rotated_pages.pdf").contains(&format!("/Rotate {angle}")));
    }
}

#[test]
fn rotation_and_page_sizes_are_read_back() {
    let generated = files();
    let rotated = lopdf::Document::load_mem(&generated["rotated_pages.pdf"]).unwrap();
    let angles: Vec<i64> = rotated
        .get_pages()
        .values()
        .map(|id| {
            rotated
                .get_dictionary(*id)
                .unwrap()
                .get(b"Rotate")
                .unwrap()
                .as_i64()
                .unwrap()
        })
        .collect();
    assert_eq!(angles, vec![0, 90, 180, 270]);

    let sized = lopdf::Document::load_mem(&generated["mixed_page_sizes.pdf"]).unwrap();
    let mut boxes: Vec<Vec<i64>> = sized
        .get_pages()
        .values()
        .map(|id| {
            let media_box = sized.get_dictionary(*id).unwrap().get(b"MediaBox").unwrap();
            media_box
                .as_array()
                .unwrap()
                .iter()
                .map(|n| n.as_i64().unwrap())
                .collect()
        })
        .collect();
    boxes.dedup();
    assert!(boxes.len() >= 5, "pages must have mixed sizes");
}

#[test]
fn malformed_files_are_really_malformed() {
    let generated = files();
    let broken_structure = [
        "malformed_truncated_one_page.pdf",
        "malformed_truncated_hundred_pages.pdf",
        "malformed_xref_offsets.pdf",
        "malformed_startxref.pdf",
        "malformed_garbage_bytes.pdf",
        "malformed_garbage_after_header.pdf",
        "malformed_zero_length.pdf",
    ];
    for name in broken_structure {
        assert!(
            verify::check_structure(&generated[name]).is_err(),
            "{name} should fail the structure check"
        );
    }
    for name in ["malformed_garbage_bytes.pdf", "malformed_zero_length.pdf"] {
        assert!(
            lopdf::Document::load_mem(&generated[name]).is_err(),
            "{name} should not open"
        );
    }
}

#[test]
fn hostile_files_have_valid_xref_but_hostile_content() {
    let generated = files();
    for name in [
        "malformed_deep_arrays.pdf",
        "malformed_deep_page_tree.pdf",
        "malformed_cyclic_page_tree.pdf",
    ] {
        verify::check_structure(&generated[name]).unwrap_or_else(|e| panic!("{name}: {e}"));
    }
    let arrays = String::from_utf8_lossy(&generated["malformed_deep_arrays.pdf"]).into_owned();
    assert!(arrays.contains(&"[".repeat(100_000)));
    let tree = String::from_utf8_lossy(&generated["malformed_deep_page_tree.pdf"]).into_owned();
    assert_eq!(tree.matches("/Type /Pages").count(), 3000);
    let cycle = String::from_utf8_lossy(&generated["malformed_cyclic_page_tree.pdf"]).into_owned();
    assert!(cycle.contains("/Kids [2 0 R 3 0 R]"));
}

#[test]
fn ten_megabyte_file_is_generated_and_valid() {
    let dir = std::env::temp_dir().join("swiftpdf-corpus-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // Only the 10 MB file: the 1,000-page file is too big for a unit test.
    let path = dir.join(large::TEN_MB_NAME);
    large::write_ten_mb(&path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert!(
        bytes.len() >= 10 * 1024 * 1024,
        "only {} bytes",
        bytes.len()
    );
    verify::check_structure(&bytes).unwrap();
    assert_eq!(page_count(&bytes), 10);
    std::fs::remove_dir_all(&dir).unwrap();
}

/// Run with `cargo test -p swiftpdf-corpus-gen -- --ignored` after `corpus-gen large`.
#[test]
#[ignore = "needs the large files from `corpus-gen large` (about 300 MB)"]
fn on_demand_large_files_are_valid() {
    let dir = std::env::var("SWIFTPDF_CORPUS_DIR").unwrap_or(r"D:\dev\swiftpdf-corpus".into());
    for (name, pages, min_bytes) in [
        (large::TEN_MB_NAME, 10, 10 * 1024 * 1024),
        (large::THOUSAND_PAGE_NAME, 1000, 300 * 1024 * 1024),
    ] {
        let bytes = std::fs::read(PathBuf::from(&dir).join(name)).unwrap();
        assert!(
            bytes.len() >= min_bytes,
            "{name} has only {} bytes",
            bytes.len()
        );
        verify::check_structure(&bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(page_count(&bytes), pages, "{name}");
    }
}
