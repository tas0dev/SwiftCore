use std::env;
use std::fs;
use std::path::PathBuf;

const GLYPH_COUNT: usize = 0x10000;
const GLYPH_ROWS: usize = 16;
const GLYPH_RECORD_SIZE: usize = 2 + GLYPH_ROWS * 2;

fn parse_hex_u16(s: &str) -> u16 {
    let mut value: u16 = 0;
    for b in s.bytes() {
        value = value.saturating_mul(16);
        value = value.saturating_add(match b {
            b'0'..=b'9' => (b - b'0') as u16,
            b'a'..=b'f' => (b - b'a' + 10) as u16,
            b'A'..=b'F' => (b - b'A' + 10) as u16,
            _ => 0,
        });
    }
    value
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let bdf_path = PathBuf::from(manifest_dir).join("src/core/unifont_jp-17.0.03.bdf");
    println!("cargo:rerun-if-changed={}", bdf_path.display());

    let text = fs::read_to_string(&bdf_path).expect("failed to read BDF file");

    let mut font_w: usize = 8;
    let mut font_h: usize = 16;

    let mut bytes = vec![0u8; 2 + GLYPH_COUNT * GLYPH_RECORD_SIZE];

    let mut in_glyph = false;
    let mut in_bitmap = false;
    let mut encoding: i32 = -1;
    let mut width: usize = 0;
    let mut height: usize = 0;
    let mut row: usize = 0;

    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("FONTBOUNDINGBOX ") {
            let mut parts = line.split_whitespace();
            let _ = parts.next();
            if let (Some(w), Some(h)) = (parts.next(), parts.next()) {
                if let (Ok(w), Ok(h)) = (w.parse::<usize>(), h.parse::<usize>()) {
                    if w > 0 && h > 0 {
                        font_w = w.min(16);
                        font_h = h.min(16);
                    }
                }
            }
            continue;
        }

        if line.starts_with("STARTCHAR") {
            in_glyph = true;
            in_bitmap = false;
            encoding = -1;
            width = 0;
            height = 0;
            row = 0;
            continue;
        }

        if line.starts_with("ENDCHAR") {
            if encoding >= 0 && (encoding as usize) < GLYPH_COUNT {
                let base = 2 + (encoding as usize) * GLYPH_RECORD_SIZE;
                bytes[base] = width.min(16) as u8;
                bytes[base + 1] = height.min(16) as u8;
            }
            in_glyph = false;
            in_bitmap = false;
            continue;
        }

        if !in_glyph {
            continue;
        }

        if line.starts_with("ENCODING ") {
            let mut parts = line.split_whitespace();
            let _ = parts.next();
            if let Some(enc) = parts.next() {
                if let Ok(v) = enc.parse::<i32>() {
                    encoding = v;
                }
            }
            continue;
        }

        if line.starts_with("BBX ") {
            let mut parts = line.split_whitespace();
            let _ = parts.next();
            if let (Some(w), Some(h)) = (parts.next(), parts.next()) {
                if let (Ok(w), Ok(h)) = (w.parse::<usize>(), h.parse::<usize>()) {
                    width = w;
                    height = h;
                }
            }
            continue;
        }

        if line == "BITMAP" {
            in_bitmap = true;
            row = 0;
            continue;
        }

        if in_bitmap {
            if encoding >= 0 && (encoding as usize) < GLYPH_COUNT && row < GLYPH_ROWS {
                let mut value = parse_hex_u16(line);
                let w = width.min(16);
                if w > 0 && w < 16 {
                    value <<= 16 - w;
                }
                let base = 2 + (encoding as usize) * GLYPH_RECORD_SIZE;
                let offset = base + 2 + row * 2;
                let [lo, hi] = value.to_le_bytes();
                bytes[offset] = lo;
                bytes[offset + 1] = hi;
            }
            row += 1;
        }
    }

    bytes[0] = font_w as u8;
    bytes[1] = font_h as u8;

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let out_path = out_dir.join("unifont.bin");
    fs::write(out_path, bytes).expect("failed to write unifont.bin");
}
