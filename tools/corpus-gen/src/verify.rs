//! A strict structural check: startxref, xref table and trailer must be consistent.

fn find_last(bytes: &[u8], needle: &[u8]) -> Option<usize> {
    if bytes.len() < needle.len() {
        return None;
    }
    (0..=bytes.len() - needle.len())
        .rev()
        .find(|&start| &bytes[start..start + needle.len()] == needle)
}

fn read_number(bytes: &[u8], start: usize) -> Result<(u64, usize), String> {
    let mut end = start;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    let text = std::str::from_utf8(&bytes[start..end]).map_err(|e| e.to_string())?;
    let value = text
        .parse()
        .map_err(|_| format!("number expected at byte {start}"))?;
    Ok((value, end))
}

fn skip_whitespace(bytes: &[u8], mut position: usize) -> usize {
    while position < bytes.len() && bytes[position].is_ascii_whitespace() {
        position += 1;
    }
    position
}

/// Checks that startxref points at an xref table whose in-use entries point at the right objects.
/// Returns the number of in-use objects.
pub fn check_structure(bytes: &[u8]) -> Result<usize, String> {
    if !bytes.starts_with(b"%PDF-") {
        return Err("missing %PDF- header".into());
    }
    let marker = find_last(bytes, b"startxref").ok_or("missing startxref")?;
    let (xref_position, _) = read_number(bytes, skip_whitespace(bytes, marker + 9))?;
    let xref_position = xref_position as usize;
    if xref_position + 4 > bytes.len() || &bytes[xref_position..xref_position + 4] != b"xref" {
        return Err(format!(
            "startxref {xref_position} does not point at an xref table"
        ));
    }

    let mut position = skip_whitespace(bytes, xref_position + 4);
    let mut in_use = 0;
    loop {
        if bytes[position..].starts_with(b"trailer") {
            break;
        }
        let (first_id, after_first) = read_number(bytes, position)?;
        let (count, after_count) = read_number(bytes, skip_whitespace(bytes, after_first))?;
        position = skip_whitespace(bytes, after_count);
        for entry in 0..count {
            let end = position + 20;
            if end > bytes.len() {
                return Err("xref table is cut off".into());
            }
            if bytes[position + 17] == b'n' {
                let (offset, _) = read_number(bytes, position)?;
                let expected = format!("{} 0 obj", first_id + entry);
                let offset = offset as usize;
                if !bytes[offset.min(bytes.len())..].starts_with(expected.as_bytes()) {
                    return Err(format!(
                        "object {} is not at offset {offset}",
                        first_id + entry
                    ));
                }
                in_use += 1;
            }
            position = end;
        }
    }
    let trailer = String::from_utf8_lossy(&bytes[position..]);
    if !trailer.contains("/Root") {
        return Err("trailer has no /Root".into());
    }
    Ok(in_use)
}
