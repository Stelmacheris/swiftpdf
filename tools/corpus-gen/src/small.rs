//! The small corpus files that are committed to the repository.

use crate::content::*;
use crate::document::*;
use crate::malformed;
use crate::writer::{escape, Obj};

const LINKED_PAGE_COUNT: usize = 6;

pub fn one_page_text() -> Vec<u8> {
    let lines = filler_lines(20, 1);
    let page = PageSpec::letter(text_page("One page of text", &lines));
    document_bytes(&[page], "", Vec::new())
}

/// 100 pages cycling through text, vector art and images.
pub fn hundred_pages_mixed() -> Vec<u8> {
    let first_image = first_free_id(100);
    let images: Vec<(u32, Obj)> = (0..4)
        .map(|n| {
            let obj = Obj::Stream {
                dict: "/Type /XObject /Subtype /Image /Width 96 /Height 96 /ColorSpace /DeviceRGB /BitsPerComponent 8".into(),
                data: rgb_gradient(96, 96, n),
            };
            (first_image + n as u32, obj)
        })
        .collect();

    let mut pages = Vec::new();
    for index in 0..100 {
        let title = format!("Page {} of 100", index + 1);
        let mut page = match index % 3 {
            0 => PageSpec::letter(text_page(&title, &filler_lines(30, index))),
            1 => PageSpec::letter(vector_page(&title, index)),
            _ => PageSpec::letter(image_page(&title, "Im0")),
        };
        if index % 3 == 2 {
            page.xobjects = format!("/Im0 {} 0 R", first_image + (index as u32 % 4));
        }
        pages.push(page);
    }
    document_bytes(&pages, "", images)
}

fn destination(page_index: usize) -> String {
    format!("[{} 0 R /XYZ 0 792 0]", page_id(page_index))
}

/// Outline (nested) plus internal, named and external links.
pub fn outline_and_links() -> Vec<u8> {
    let base = first_free_id(LINKED_PAGE_COUNT);
    let root = base;
    let (chapter1, section11, section12) = (base + 1, base + 2, base + 3);
    let (chapter2, chapter3) = (base + 4, base + 5);
    let (link_page, link_action, link_uri, link_named) = (base + 6, base + 7, base + 8, base + 9);

    let item = |title: &str, parent: u32, page: usize, links: &str| {
        Obj::Dict(format!(
            "<< /Title ({title}) /Parent {parent} 0 R /Dest {} {links} >>",
            destination(page)
        ))
    };
    let outline = vec![
        (
            root,
            Obj::Dict(format!(
                "<< /Type /Outlines /First {chapter1} 0 R /Last {chapter3} 0 R /Count 5 >>"
            )),
        ),
        (
            chapter1,
            item(
                "Chapter 1",
                root,
                0,
                &format!(
                    "/Next {chapter2} 0 R /First {section11} 0 R /Last {section12} 0 R /Count 2"
                ),
            ),
        ),
        (
            section11,
            item(
                "Section 1.1",
                chapter1,
                0,
                &format!("/Next {section12} 0 R"),
            ),
        ),
        (
            section12,
            item(
                "Section 1.2",
                chapter1,
                1,
                &format!("/Prev {section11} 0 R"),
            ),
        ),
        (
            chapter2,
            item(
                "Chapter 2",
                root,
                2,
                &format!("/Prev {chapter1} 0 R /Next {chapter3} 0 R"),
            ),
        ),
        (
            chapter3,
            item("Chapter 3", root, 4, &format!("/Prev {chapter2} 0 R")),
        ),
    ];

    let link = |rect: &str, target: &str| {
        Obj::Dict(format!(
            "<< /Type /Annot /Subtype /Link /Rect [{rect}] /Border [0 0 1] {target} >>"
        ))
    };
    let links = vec![
        (
            link_page,
            link(
                "72 640 300 660",
                &format!("/Dest [{} 0 R /Fit]", page_id(2)),
            ),
        ),
        (
            link_action,
            link(
                "72 610 300 630",
                &format!("/A << /S /GoTo /D {} >>", destination(3)),
            ),
        ),
        (
            link_uri,
            link(
                "72 580 300 600",
                "/A << /S /URI /URI (https://example.com/) >>",
            ),
        ),
        (link_named, link("72 550 300 570", "/Dest (chapter2)")),
    ];

    let mut pages = Vec::new();
    for index in 0..LINKED_PAGE_COUNT {
        let title = format!("Linked page {}", index + 1);
        let mut content = text_page(&title, &filler_lines(5, index));
        if index == 0 {
            content.push_str(
                "BT /F1 12 Tf 72 645 Td (Internal link to page 3) Tj 0 -30 Td (GoTo action to page 4) Tj \
                 0 -30 Td (External link: example.com) Tj 0 -30 Td (Named destination: chapter2) Tj ET\n",
            );
        }
        let mut page = PageSpec::letter(content);
        if index == 0 {
            page.extra = format!(
                "/Annots [{link_page} 0 R {link_action} 0 R {link_uri} 0 R {link_named} 0 R]"
            );
        }
        pages.push(page);
    }
    let catalog = format!(
        "/Outlines {root} 0 R /PageMode /UseOutlines /Names << /Dests << /Names [(chapter2) {}] >> >>",
        destination(2)
    );
    document_bytes(&pages, &catalog, outline.into_iter().chain(links))
}

fn empty_appearance() -> Obj {
    Obj::Stream {
        dict: "/Type /XObject /Subtype /Form /BBox [0 0 20 20]".into(),
        data: b"q Q".to_vec(),
    }
}

/// AcroForm with text, checkbox, radio group, combo box and list box. With `xfa` it is also XFA-flagged.
fn form_document(xfa: bool) -> Vec<u8> {
    let base = first_free_id(1);
    let appearance = base;
    let (text, check, radio) = (base + 1, base + 2, base + 3);
    let (radio_small, radio_large) = (base + 4, base + 5);
    let (combo, list, acroform) = (base + 6, base + 7, base + 8);
    let (xfa_template, xfa_data) = (base + 9, base + 10);
    let page = page_id(0);

    let widget = |rect: &str, rest: &str| {
        format!("<< /Type /Annot /Subtype /Widget /F 4 /P {page} 0 R /Rect [{rect}] {rest} >>")
    };
    let check_appearance = format!("/AP << /N << /On {appearance} 0 R /Off {appearance} 0 R >> >>");
    let small_appearance =
        format!("/AP << /N << /Small {appearance} 0 R /Off {appearance} 0 R >> >>");
    let large_appearance =
        format!("/AP << /N << /Large {appearance} 0 R /Off {appearance} 0 R >> >>");
    let mut extras = vec![
        (appearance, empty_appearance()),
        (
            text,
            Obj::Dict(widget(
                "72 650 300 674",
                "/FT /Tx /T (full_name) /V (Ada Lovelace) /DA (/Helv 12 Tf 0 g)",
            )),
        ),
        (
            check,
            Obj::Dict(widget(
                "72 610 92 630",
                &format!("/FT /Btn /T (agree) /V /Off /AS /Off {check_appearance}"),
            )),
        ),
        (
            radio,
            Obj::Dict(format!(
                "<< /FT /Btn /Ff 32768 /T (size) /V /Small /Kids [{radio_small} 0 R {radio_large} 0 R] >>"
            )),
        ),
        (
            radio_small,
            Obj::Dict(widget(
                "72 570 92 590",
                &format!("/Parent {radio} 0 R /AS /Small {small_appearance}"),
            )),
        ),
        (
            radio_large,
            Obj::Dict(widget(
                "120 570 140 590",
                &format!("/Parent {radio} 0 R /AS /Off {large_appearance}"),
            )),
        ),
        (
            combo,
            Obj::Dict(widget(
                "72 530 300 552",
                "/FT /Ch /Ff 131072 /T (color) /Opt [(Red) (Green) (Blue)] /V (Green) /DA (/Helv 12 Tf 0 g)",
            )),
        ),
        (
            list,
            Obj::Dict(widget(
                "72 420 300 510",
                "/FT /Ch /T (fruit) /Opt [(Apple) (Banana) (Cherry) (Date)] /V (Banana) /DA (/Helv 12 Tf 0 g)",
            )),
        ),
    ];

    let xfa_entry = if xfa {
        format!("/XFA [(template) {xfa_template} 0 R (datasets) {xfa_data} 0 R]")
    } else {
        String::new()
    };
    let fields = format!("{text} 0 R {check} 0 R {radio} 0 R {combo} 0 R {list} 0 R");
    let acroform_dict = format!(
        "<< /Fields [{fields}] /NeedAppearances true /DA (/Helv 12 Tf 0 g) \
         /DR << /Font << /Helv {FONT_ID} 0 R >> >> {xfa_entry} >>"
    );
    extras.push((acroform, Obj::Dict(acroform_dict)));
    if xfa {
        let template = "<?xml version=\"1.0\"?><xdp:xdp xmlns:xdp=\"http://ns.adobe.com/xdp/\"><template xmlns=\"http://www.xfa.org/schema/xfa-template/3.3/\"><subform name=\"form1\"><field name=\"full_name\"/></subform></template></xdp:xdp>";
        let datasets = "<?xml version=\"1.0\"?><xfa:datasets xmlns:xfa=\"http://www.xfa.org/schema/xfa-data/1.0/\"><xfa:data><form1><full_name>Ada Lovelace</full_name></form1></xfa:data></xfa:datasets>";
        extras.push((
            xfa_template,
            Obj::Stream {
                dict: String::new(),
                data: template.as_bytes().to_vec(),
            },
        ));
        extras.push((
            xfa_data,
            Obj::Stream {
                dict: String::new(),
                data: datasets.as_bytes().to_vec(),
            },
        ));
    }

    let title = if xfa {
        "XFA-flagged form"
    } else {
        "AcroForm: text, checkbox, radio, combo, list"
    };
    let mut sheet = PageSpec::letter(text_page(title, &[]));
    sheet.extra = format!(
        "/Annots [{text} 0 R {check} 0 R {radio_small} 0 R {radio_large} 0 R {combo} 0 R {list} 0 R]"
    );
    document_bytes(&[sheet], &format!("/AcroForm {acroform} 0 R"), extras)
}

pub fn acroform() -> Vec<u8> {
    form_document(false)
}

pub fn xfa_form() -> Vec<u8> {
    form_document(true)
}

/// Two embedded files in the name tree, plus a file-attachment annotation.
pub fn attachments() -> Vec<u8> {
    let base = first_free_id(1);
    let (text_data, csv_data) = (base, base + 1);
    let (text_spec, csv_spec, annotation) = (base + 2, base + 3, base + 4);
    let hello = "Hello from an embedded file.\n";
    let table = "id,name\n1,alpha\n2,beta\n";

    let file_stream = |subtype: &str, text: &str| Obj::Stream {
        dict: format!(
            "/Type /EmbeddedFile /Subtype /{subtype} /Params << /Size {} >>",
            text.len()
        ),
        data: text.as_bytes().to_vec(),
    };
    let file_spec = |name: &str, stream: u32| {
        Obj::Dict(format!(
            "<< /Type /Filespec /F ({0}) /UF ({0}) /Desc (Sample {0}) /EF << /F {stream} 0 R >> >>",
            escape(name)
        ))
    };
    let extras = vec![
        (text_data, file_stream("text#2Fplain", hello)),
        (csv_data, file_stream("text#2Fcsv", table)),
        (text_spec, file_spec("hello.txt", text_data)),
        (csv_spec, file_spec("data.csv", csv_data)),
        (
            annotation,
            Obj::Dict(format!(
                "<< /Type /Annot /Subtype /FileAttachment /Rect [72 600 92 620] /Name /Paperclip /Contents (hello.txt) /FS {text_spec} 0 R >>"
            )),
        ),
    ];
    let body = ["Two embedded files: hello.txt and data.csv.".to_string()];
    let mut page = PageSpec::letter(text_page("Document with attachments", &body));
    page.extra = format!("/Annots [{annotation} 0 R]");
    let catalog = format!(
        "/Names << /EmbeddedFiles << /Names [(data.csv) {csv_spec} 0 R (hello.txt) {text_spec} 0 R] >> >>"
    );
    document_bytes(&[page], &catalog, extras)
}

/// Pages rotated 0, 90, 180 and 270 degrees.
pub fn rotated_pages() -> Vec<u8> {
    let body = ["The top edge of the unrotated page is here.".to_string()];
    let pages: Vec<PageSpec> = [0, 90, 180, 270]
        .iter()
        .map(|&angle| {
            let mut page = PageSpec::letter(text_page(&format!("Rotate {angle}"), &body));
            page.rotate = angle;
            page
        })
        .collect();
    document_bytes(&pages, "", Vec::new())
}

/// Letter, A4, A3 landscape, tiny, poster, and a page with a smaller CropBox.
pub fn mixed_page_sizes() -> Vec<u8> {
    let sizes = [
        (612, 792, "Letter"),
        (595, 842, "A4"),
        (1191, 842, "A3 landscape"),
        (200, 200, "Tiny 200x200"),
        (2000, 3000, "Poster 2000x3000"),
    ];
    let mut pages: Vec<PageSpec> = sizes
        .iter()
        .map(|&(width, height, name)| PageSpec::new(width, height, text_page(name, &[])))
        .collect();
    let mut cropped = PageSpec::new(612, 792, text_page("CropBox 50 50 400 400", &[]));
    cropped.extra = "/CropBox [50 50 400 400]".into();
    pages.push(cropped);
    document_bytes(&pages, "", Vec::new())
}

/// Every committed corpus file, as (file name, bytes).
pub fn small_files() -> Vec<(&'static str, Vec<u8>)> {
    let one_page = one_page_text();
    let hundred = hundred_pages_mixed();
    let mut files = vec![
        ("one_page_text.pdf", one_page.clone()),
        ("hundred_pages_mixed.pdf", hundred.clone()),
        ("outline_and_links.pdf", outline_and_links()),
        ("acroform.pdf", acroform()),
        ("xfa_form.pdf", xfa_form()),
        ("attachments.pdf", attachments()),
        ("rotated_pages.pdf", rotated_pages()),
        ("mixed_page_sizes.pdf", mixed_page_sizes()),
    ];
    files.extend(malformed::malformed_files(&one_page, &hundred));
    files
}
