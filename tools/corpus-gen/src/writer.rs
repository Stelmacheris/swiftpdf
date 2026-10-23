//! A tiny dependency-free PDF writer. It writes objects as it goes, so huge files never sit in memory.

use std::collections::BTreeMap;
use std::io::{self, Write};

/// One indirect object body.
pub enum Obj {
    /// A complete dictionary or array, such as `<< /A 1 >>`.
    Dict(String),
    /// A stream: `dict` holds the dictionary entries without the outer `<< >>` and without `/Length`.
    Stream { dict: String, data: Vec<u8> },
}

pub struct PdfWriter<W: Write> {
    out: W,
    position: u64,
    offsets: BTreeMap<u32, u64>,
}

impl<W: Write> PdfWriter<W> {
    pub fn new(out: W) -> io::Result<Self> {
        let mut writer = PdfWriter {
            out,
            position: 0,
            offsets: BTreeMap::new(),
        };
        writer.put(b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n")?;
        Ok(writer)
    }

    fn put(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.out.write_all(bytes)?;
        self.position += bytes.len() as u64;
        Ok(())
    }

    pub fn write_obj(&mut self, id: u32, obj: &Obj) -> io::Result<()> {
        self.offsets.insert(id, self.position);
        self.put(format!("{id} 0 obj\n").as_bytes())?;
        match obj {
            Obj::Dict(body) => self.put(body.as_bytes())?,
            Obj::Stream { dict, data } => {
                self.put(format!("<< {dict} /Length {} >>\nstream\n", data.len()).as_bytes())?;
                self.put(data)?;
                self.put(b"\nendstream")?;
            }
        }
        self.put(b"\nendobj\n")
    }

    /// Writes the xref table and trailer, and returns the underlying writer.
    pub fn finish(mut self, root: u32) -> io::Result<W> {
        let xref_position = self.position;
        let last_id = self.offsets.keys().next_back().copied().unwrap_or(0);
        let mut table = format!("xref\n0 {}\n0000000000 65535 f \n", last_id + 1);
        for id in 1..=last_id {
            match self.offsets.get(&id) {
                Some(offset) => table.push_str(&format!("{offset:010} 00000 n \n")),
                None => table.push_str("0000000000 65535 f \n"),
            }
        }
        table.push_str(&format!(
            "trailer\n<< /Size {} /Root {root} 0 R >>\nstartxref\n{xref_position}\n%%EOF\n",
            last_id + 1
        ));
        self.put(table.as_bytes())?;
        self.out.flush()?;
        Ok(self.out)
    }
}

/// Escapes text for use inside a PDF literal string (without the parentheses).
pub fn escape(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}
