//! Page content helpers: text, vector art, and deterministic image data.

use crate::writer::escape;

/// A title and a few lines of body text.
pub fn text_page(title: &str, lines: &[String]) -> String {
    let mut content = format!(
        "BT /F2 24 Tf 72 720 Td ({}) Tj /F1 12 Tf 0 -36 Td 16 TL\n",
        escape(title)
    );
    for line in lines {
        content.push_str(&format!("({}) Tj T*\n", escape(line)));
    }
    content.push_str("ET\n");
    content
}

pub fn filler_lines(count: usize, seed: usize) -> Vec<String> {
    (0..count)
        .map(|n| {
            format!(
                "Line {} of original sample text, seed {}. The quick brown fox jumps over the lazy dog.",
                n + 1,
                seed
            )
        })
        .collect()
}

/// Rectangles, lines and curves in several colors.
pub fn vector_page(title: &str, seed: usize) -> String {
    let mut content = format!("BT /F2 18 Tf 72 740 Td ({}) Tj ET\n", escape(title));
    for n in 0..8 {
        let shade = ((n + seed) % 8) as f32 / 8.0;
        let x = 60 + n * 60;
        let height = 40 + n * 30;
        content.push_str(&format!(
            "{shade:.2} {:.2} 0.50 rg {x} 400 50 {height} re f\n",
            1.0 - shade
        ));
    }
    content.push_str("2 w 0 0 1 RG 72 300 m 200 500 400 100 540 300 c S\n");
    content.push_str("0.5 w 0 G 72 250 m 540 250 l S\n");
    content.push_str("1 0 0 RG 306 200 m 356 200 356 150 306 150 c 256 150 256 200 306 200 c S\n");
    content
}

pub fn image_page(title: &str, image_name: &str) -> String {
    format!(
        "BT /F2 18 Tf 72 740 Td ({}) Tj ET\nq 300 0 0 300 156 300 cm /{image_name} Do Q\n",
        escape(title)
    )
}

/// A small RGB gradient image; `variant` changes the color mix.
pub fn rgb_gradient(width: usize, height: usize, variant: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(width * height * 3);
    for y in 0..height {
        for x in 0..width {
            let red = (x * 255 / width) as u8;
            let green = (y * 255 / height) as u8;
            let blue = ((x + y + variant * 64) % 256) as u8;
            data.extend_from_slice(&[red, green, blue]);
        }
    }
    data
}

/// Deterministic pseudo-random bytes (xorshift64). Used for noise images and garbage files.
pub fn noise(length: usize, seed: u64) -> Vec<u8> {
    let mut state = seed | 1;
    let mut data = Vec::with_capacity(length + 8);
    while data.len() < length {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        data.extend_from_slice(&state.to_le_bytes());
    }
    data.truncate(length);
    data
}
