//! Large files that are generated on demand and never committed. They stream to disk page by page.

use crate::content::noise;
use crate::document::*;
use crate::writer::{Obj, PdfWriter};
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;

/// 10 pages of 1,049,600 bytes of noise each: a file just over 10 MiB.
pub const TEN_MB_NAME: &str = "large_10mb.pdf";
/// 1,000 pages of 315,000 bytes each: about 300 MiB.
pub const THOUSAND_PAGE_NAME: &str = "large_1000pages_300mb.pdf";

/// Writes `page_count` pages, each showing its own noise image of `width` x `height` gray pixels.
fn write_noise_document(
    path: &Path,
    page_count: usize,
    width: usize,
    height: usize,
) -> io::Result<()> {
    let first_image = first_free_id(page_count);
    let pages: Vec<PageSpec> = (0..page_count)
        .map(|index| {
            let content = format!(
                "q 612 0 0 792 0 0 cm /Im0 Do Q\nBT /F2 28 Tf 1 1 1 rg 40 720 Td (Page {} of {}) Tj ET\n",
                index + 1,
                page_count
            );
            let mut page = PageSpec::letter(content);
            page.xobjects = format!("/Im0 {} 0 R", first_image + index as u32);
            page
        })
        .collect();
    let images = (0..page_count).map(|index| {
        let dict = format!(
            "/Type /XObject /Subtype /Image /Width {width} /Height {height} /ColorSpace /DeviceGray /BitsPerComponent 8"
        );
        let data = noise(width * height, index as u64 + 1);
        (first_image + index as u32, Obj::Stream { dict, data })
    });

    let file = BufWriter::new(File::create(path)?);
    let writer = PdfWriter::new(file)?;
    write_document(writer, &pages, "", images)?;
    Ok(())
}

pub fn write_ten_mb(path: &Path) -> io::Result<()> {
    write_noise_document(path, 10, 1024, 1025)
}

pub fn write_thousand_pages(path: &Path) -> io::Result<()> {
    write_noise_document(path, 1000, 630, 500)
}

pub fn write_large_files(dir: &Path) -> io::Result<Vec<std::path::PathBuf>> {
    std::fs::create_dir_all(dir)?;
    let ten_mb = dir.join(TEN_MB_NAME);
    write_ten_mb(&ten_mb)?;
    let thousand = dir.join(THOUSAND_PAGE_NAME);
    write_thousand_pages(&thousand)?;
    Ok(vec![ten_mb, thousand])
}
