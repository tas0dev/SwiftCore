//! フレームバッファ出力

use core::fmt;
use spin::{Mutex, Once};

/// フレームバッファ情報
static FB_INFO: Once<FramebufferInfo> = Once::new();

#[derive(Clone, Copy)]
struct FramebufferInfo {
    addr: u64,
    width: usize,
    height: usize,
    stride: usize,
}

/// フォントの最大グリフ数（BMP）
const GLYPH_COUNT: usize = 0x10000;
const GLYPH_ROWS: usize = 16;
const GLYPH_RECORD_SIZE: usize = 2 + GLYPH_ROWS * 2;
const FONT_HEADER_SIZE: usize = 2;

/// フォント（ビルド時生成バイナリ）
static FONT: Once<FontData> = Once::new();

struct FontData {
    width: usize,
    height: usize,
    data: &'static [u8],
}

fn init_font() -> FontData {
    let data: &'static [u8] =
        include_bytes!(concat!(env!("OUT_DIR"), "/unifont.bin"));
    let width = data.get(0).copied().unwrap_or(8) as usize;
    let height = data.get(1).copied().unwrap_or(16) as usize;
    FontData {
        width,
        height,
        data,
    }
}

fn glyph_record<'a>(font: &'a FontData, codepoint: u32) -> Option<&'a [u8]> {
    let idx = codepoint as usize;
    if idx >= GLYPH_COUNT {
        return None;
    }
    let base = FONT_HEADER_SIZE + idx * GLYPH_RECORD_SIZE;
    let end = base + GLYPH_RECORD_SIZE;
    if end <= font.data.len() {
        Some(&font.data[base..end])
    } else {
        None
    }
}

/// フレームバッファライター
pub struct Writer {
    column: usize,
    row: usize,
    max_cols: usize,
    max_rows: usize,
    font_width: usize,
    font_height: usize,
}

impl Writer {
    fn new(info: &FramebufferInfo, font_width: usize, font_height: usize) -> Self {
        let max_cols = info.width / font_width;
        let max_rows = info.height / font_height;
        Self {
            column: 0,
            row: 0,
            max_cols,
            max_rows,
            font_width,
            font_height,
        }
    }

    /// ピクセルを描画
    fn put_pixel(&self, x: usize, y: usize, color: u32) {
        if let Some(info) = FB_INFO.get() {
            // 境界チェック
            if x >= info.width || y >= info.height {
                return;
            }
            let offset = y * info.stride + x;
            let fb_ptr = info.addr as *mut u32;
            unsafe {
                fb_ptr.add(offset).write_volatile(color);
            }
        }
    }

    /// 文字を描画（BDFフォント）
    fn draw_char(&self, codepoint: u32, x: usize, y: usize, fg: u32, bg: u32) {
        let font = match FONT.get() {
            Some(font) => font,
            None => return,
        };

        let record = match glyph_record(font, codepoint) {
            Some(record) => record,
            None => return,
        };

        let bitmap_empty = record
            .get(2..)
            .map(|bytes| bytes.iter().all(|&b| b == 0))
            .unwrap_or(true);

        let glyph_w = match record.get(0).copied().unwrap_or(0) {
            0 => font.width,
            w => w as usize,
        };
        let glyph_h = match record.get(1).copied().unwrap_or(0) {
            0 => font.height,
            h => h as usize,
        };

        for row in 0..self.font_height {
            for col in 0..self.font_width {
                let is_set = if bitmap_empty {
                    let is_printable = (0x20..=0x7e).contains(&codepoint);
                    is_printable
                        && (row == 0
                            || row + 1 == self.font_height
                            || col == 0
                            || col + 1 == self.font_width)
                } else if row < glyph_h && col < glyph_w {
                    let offset = 2 + row * 2;
                    let lo = record.get(offset).copied().unwrap_or(0);
                    let hi = record.get(offset + 1).copied().unwrap_or(0);
                    let bits = u16::from_le_bytes([lo, hi]);
                    let mask = 1u16 << (15 - col);
                    (bits & mask) != 0
                } else {
                    false
                };
                let color = if is_set { fg } else { bg };
                self.put_pixel(x + col, y + row, color);
            }
        }
    }

    #[cfg(debug_assertions)]
    fn draw_test_pattern(&self) {
        for y in 0..8 {
            for x in 0..8 {
                self.put_pixel(x, y, 0x00FF00); // 緑
            }
        }
    }

    /// 1文字書き込み
    pub fn write_char(&mut self, ch: char) {
        if ch == '\n' {
            self.new_line();
            return;
        }

        if self.column >= self.max_cols {
            self.new_line();
        }

        let x = self.column * self.font_width;
        let y = self.row * self.font_height;
        self.draw_char(ch as u32, x, y, 0xFFFFFF, 0x000000); // 白文字、黒背景

        self.column += 1;
    }

    /// 文字列を書き込み
    pub fn write_string(&mut self, s: &str) {
        for ch in s.chars() {
            self.write_char(ch);
        }
    }

    /// 改行処理
    fn new_line(&mut self) {
        self.row += 1;
        self.column = 0;
        if self.row >= self.max_rows {
            // スクロールの代わりに画面クリア（簡易版）
            self.clear_screen();
        }
    }

    /// 画面全体をクリア
    pub fn clear_screen(&mut self) {
        if let Some(info) = FB_INFO.get() {
            let fb_ptr = info.addr as *mut u32;

            let total_pixels = info.height * info.width;
            unsafe {
                for i in 0..total_pixels {
                    fb_ptr.add(i).write_volatile(0x000000); // 黒
                }
            }
        }
        self.row = 0;
        self.column = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// グローバルライター（遅延初期化）
static WRITER: Once<Mutex<Writer>> = Once::new();

/// フレームバッファを初期化
pub fn init(addr: u64, width: usize, height: usize, stride: usize) {
    FB_INFO.call_once(|| FramebufferInfo {
        addr,
        width,
        height,
        stride,
    });

    FONT.call_once(init_font);

    if let Some(info) = FB_INFO.get() {
        let (font_w, font_h) = FONT
            .get()
            .map(|font| (font.width, font.height))
            .unwrap_or((8, 16));
        WRITER.call_once(|| Mutex::new(Writer::new(info, font_w, font_h)));

        // 画面をクリア
        if let Some(writer) = WRITER.get() {
            writer.lock().clear_screen();
            #[cfg(debug_assertions)]
            writer.lock().draw_test_pattern();
        }
    }
}

/// フレームバッファに文字列を出力（割り込み対応）
pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    if let Some(writer) = WRITER.get() {
        // 割り込みを無効化してロック取得（デッドロック防止）
        x86_64::instructions::interrupts::without_interrupts(|| {
            let _ = writer.lock().write_fmt(args);
        });
    }
}

/// フレームバッファ出力マクロ
#[macro_export]
macro_rules! vprint {
    ($($arg:tt)*) => {
        $crate::util::vga::print(format_args!($($arg)*))
    };
}

/// 改行付きフレームバッファ出力マクロ
#[macro_export]
macro_rules! vprintln {
    () => ($crate::vprint!("\n"));
    ($($arg:tt)*) => {
        $crate::vprint!("{}\n", format_args!($($arg)*))
    };
}
