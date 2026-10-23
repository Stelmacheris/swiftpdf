//! Builds ordinary documents: catalog, page tree, two fonts, and one page object plus one content stream per page.

use crate::writer::{Obj, PdfWriter};
use std::io::{self, Write};

pub const CATALOG_ID: u32 = 1;
pub const PAGES_ID: u32 = 2;
pub const FONT_ID: u32 = 3;
pub const FONT_BOLD_ID: u32 = 4;

pub fn page_id(index: usize) -> u32 {
    5 + 2 * index as u32
}

fn content_id(index: usize) -> u32 {
    6 + 2 * index as u32
}

/// The first object id that is free for extra objects (images, forms, outlines, ...).
pub fn first_free_id(page_count: usize) -> u32 {
    5 + 2 * page_count as u32
}

pub struct PageSpec {
    pub width: i32,
    pub height: i32,
    pub rotate: i32,
    pub content: String,
    /// Entries for the `/XObject` resource dictionary, such as `/Im0 12 0 R`.
    pub xobjects: String,
    /// Extra page dictionary entries, such as `/Annots [..]`.
    pub extra: String,
}

impl PageSpec {
    pub fn new(width: i32, height: i32, content: String) -> Self {
        PageSpec {
            width,
            height,
            rotate: 0,
            content,
            xobjects: String::new(),
            extra: String::new(),
        }
    }

    pub fn letter(content: String) -> Self {
        PageSpec::new(612, 792, content)
    }
}

/// Writes a complete document and returns the underlying writer.
pub fn write_document<W: Write>(
    mut writer: PdfWriter<W>,
    pages: &[PageSpec],
    catalog_extra: &str,
    extras: impl IntoIterator<Item = (u32, Obj)>,
) -> io::Result<W> {
    let catalog = format!("<< /Type /Catalog /Pages {PAGES_ID} 0 R {catalog_extra} >>");
    writer.write_obj(CATALOG_ID, &Obj::Dict(catalog))?;

    let kids: Vec<String> = (0..pages.len())
        .map(|index| format!("{} 0 R", page_id(index)))
        .collect();
    let page_tree = format!(
        "<< /Type /Pages /Kids [{}] /Count {} >>",
        kids.join(" "),
        pages.len()
    );
    writer.write_obj(PAGES_ID, &Obj::Dict(page_tree))?;

    let font = "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>";
    let font_bold =
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>";
    writer.write_obj(FONT_ID, &Obj::Dict(font.to_string()))?;
    writer.write_obj(FONT_BOLD_ID, &Obj::Dict(font_bold.to_string()))?;

    for (index, page) in pages.iter().enumerate() {
        let dict = format!(
            "<< /Type /Page /Parent {PAGES_ID} 0 R /MediaBox [0 0 {} {}] /Rotate {} \
             /Resources << /Font << /F1 {FONT_ID} 0 R /F2 {FONT_BOLD_ID} 0 R >> /XObject << {} >> >> \
             /Contents {} 0 R {} >>",
            page.width,
            page.height,
            page.rotate,
            page.xobjects,
            content_id(index),
            page.extra
        );
        writer.write_obj(page_id(index), &Obj::Dict(dict))?;
        let content = Obj::Stream {
            dict: String::new(),
            data: page.content.clone().into_bytes(),
        };
        writer.write_obj(content_id(index), &content)?;
    }

    for (id, obj) in extras {
        writer.write_obj(id, &obj)?;
    }
    writer.finish(CATALOG_ID)
}

/// Builds a document in memory.
pub fn document_bytes(
    pages: &[PageSpec],
    catalog_extra: &str,
    extras: impl IntoIterator<Item = (u32, Obj)>,
) -> Vec<u8> {
    let writer = PdfWriter::new(Vec::new()).expect("writing to memory cannot fail");
    write_document(writer, pages, catalog_extra, extras).expect("writing to memory cannot fail")
}
