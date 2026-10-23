//! Malformed and hostile files. Every one must make a reader return an error or cope, never crash.

use crate::content::{noise, text_page};
use crate::document::*;
use crate::writer::{Obj, PdfWriter};

const ARRAY_DEPTH: usize = 100_000;
const PAGE_TREE_DEPTH: u32 = 3_000;

fn find_last(bytes: &[u8], needle: &[u8]) -> usize {
    (0..=bytes.len() - needle.len())
        .rev()
        .find(|&start| &bytes[start..start + needle.len()] == needle)
        .expect("marker present in generated file")
}

/// Cuts the file off in the middle of the last object, so the xref and trailer are gone.
fn truncated_before_xref(valid: &[u8]) -> Vec<u8> {
    let xref_start = find_last(valid, b"\nxref\n");
    valid[..xref_start - 30].to_vec()
}

/// Keeps the xref table but moves every in-use offset by 13 bytes.
fn broken_xref_offsets(valid: &[u8]) -> Vec<u8> {
    let xref_start = find_last(valid, b"\nxref\n") + 1;
    let header_end = xref_start
        + valid[xref_start..]
            .iter()
            .position(|&b| b == b'\n')
            .unwrap()
        + 1;
    let count_end = header_end
        + valid[header_end..]
            .iter()
            .position(|&b| b == b'\n')
            .unwrap()
        + 1;
    let mut broken = valid.to_vec();
    let mut entry_start = count_end;
    while &broken[entry_start..entry_start + 7] != b"trailer" {
        if broken[entry_start + 17] == b'n' {
            let offset: u64 = std::str::from_utf8(&broken[entry_start..entry_start + 10])
                .unwrap()
                .parse()
                .unwrap();
            let moved = format!("{:010}", offset + 13);
            broken[entry_start..entry_start + 10].copy_from_slice(moved.as_bytes());
        }
        entry_start += 20;
    }
    broken
}

/// Points startxref far beyond the end of the file.
fn broken_startxref(valid: &[u8]) -> Vec<u8> {
    let marker = find_last(valid, b"startxref\n");
    let mut broken = valid[..marker].to_vec();
    broken.extend_from_slice(b"startxref\n99999999\n%%EOF\n");
    broken
}

fn write_all_objects(objects: Vec<(u32, Obj)>) -> Vec<u8> {
    let mut writer = PdfWriter::new(Vec::new()).expect("memory write");
    for (id, obj) in &objects {
        writer.write_obj(*id, obj).expect("memory write");
    }
    writer.finish(CATALOG_ID).expect("memory write")
}

/// A valid document with one object that is an array nested 100,000 deep (stack-overflow bait).
fn deep_nesting_arrays() -> Vec<u8> {
    let page = PageSpec::letter(text_page("Deep array nesting", &[]));
    let deep = format!("{}{}", "[".repeat(ARRAY_DEPTH), "]".repeat(ARRAY_DEPTH));
    let id = first_free_id(1);
    document_bytes(
        &[page],
        &format!("/Deep {id} 0 R"),
        vec![(id, Obj::Dict(deep))],
    )
}

/// A page tree where each Pages node has exactly one child Pages node, 3,000 levels down to one page.
fn deep_nesting_page_tree() -> Vec<u8> {
    let first_node = PAGES_ID;
    let leaf = first_node + PAGE_TREE_DEPTH;
    let mut objects = vec![(
        CATALOG_ID,
        Obj::Dict(format!("<< /Type /Catalog /Pages {first_node} 0 R >>")),
    )];
    for level in 0..PAGE_TREE_DEPTH {
        let id = first_node + level;
        let parent = if level == 0 {
            String::new()
        } else {
            format!("/Parent {} 0 R", id - 1)
        };
        objects.push((
            id,
            Obj::Dict(format!(
                "<< /Type /Pages {parent} /Kids [{} 0 R] /Count 1 >>",
                id + 1
            )),
        ));
    }
    objects.push((
        leaf,
        Obj::Dict(format!(
            "<< /Type /Page /Parent {} 0 R /MediaBox [0 0 612 792] >>",
            leaf - 1
        )),
    ));
    write_all_objects(objects)
}

/// A page tree whose root lists itself as a child.
fn cyclic_page_tree() -> Vec<u8> {
    write_all_objects(vec![
        (
            CATALOG_ID,
            Obj::Dict(format!("<< /Type /Catalog /Pages {PAGES_ID} 0 R >>")),
        ),
        (
            PAGES_ID,
            Obj::Dict(format!(
                "<< /Type /Pages /Kids [{PAGES_ID} 0 R 3 0 R] /Count 2 >>"
            )),
        ),
        (
            3,
            Obj::Dict(format!(
                "<< /Type /Page /Parent {PAGES_ID} 0 R /MediaBox [0 0 612 792] >>"
            )),
        ),
    ])
}

/// The malformed set, as (file name, bytes). Built from the valid one-page and 100-page files.
pub fn malformed_files(one_page: &[u8], hundred_pages: &[u8]) -> Vec<(&'static str, Vec<u8>)> {
    let mut after_header = b"%PDF-1.7\n".to_vec();
    after_header.extend(noise(2048, 7));
    vec![
        (
            "malformed_truncated_one_page.pdf",
            truncated_before_xref(one_page),
        ),
        (
            "malformed_truncated_hundred_pages.pdf",
            hundred_pages[..hundred_pages.len() / 2].to_vec(),
        ),
        ("malformed_xref_offsets.pdf", broken_xref_offsets(one_page)),
        ("malformed_startxref.pdf", broken_startxref(one_page)),
        ("malformed_garbage_bytes.pdf", noise(4096, 42)),
        ("malformed_garbage_after_header.pdf", after_header),
        ("malformed_zero_length.pdf", Vec::new()),
        ("malformed_deep_arrays.pdf", deep_nesting_arrays()),
        ("malformed_deep_page_tree.pdf", deep_nesting_page_tree()),
        ("malformed_cyclic_page_tree.pdf", cyclic_page_tree()),
    ]
}
