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

/// 8x16 ビットマップフォント（簡易版）
const FONT_HEIGHT: usize = 16;
const FONT_WIDTH: usize = 8;

/// フレームバッファライター
pub struct Writer {
    column: usize,
    row: usize,
    max_cols: usize,
    max_rows: usize,
}

impl Writer {
    fn new(info: &FramebufferInfo) -> Self {
        let max_cols = info.width / FONT_WIDTH;
        let max_rows = info.height / FONT_HEIGHT;
        Self {
            column: 0,
            row: 0,
            max_cols,
            max_rows,
        }
    }

    /// ピクセルを描画
    fn put_pixel(&self, x: usize, y: usize, color: u32) {
        if let Some(info) = FB_INFO.get() {
            let offset = y * info.stride + x;
            let fb_ptr = info.addr as *mut u32;
            unsafe {
                fb_ptr.add(offset).write_volatile(color);
            }
        }
    }

    /// 文字を描画（簡易版：矩形で代用）
    fn draw_char(&self, ch: u8, x: usize, y: usize, fg: u32, bg: u32) {
        // 簡易実装：背景色で矩形を描画
        for row in 0..FONT_HEIGHT {
            for col in 0..FONT_WIDTH {
                // 文字の輪郭を簡易的に表現（実際のフォントデータは省略）
                let is_char = ch != b' '
                    && (row == 0 || row == FONT_HEIGHT - 1 || col == 0 || col == FONT_WIDTH - 1);
                let color = if is_char { fg } else { bg };
                self.put_pixel(x + col, y + row, color);
            }
        }
    }

    /// 1バイト書き込み
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column >= self.max_cols {
                    self.new_line();
                }

                let x = self.column * FONT_WIDTH;
                let y = self.row * FONT_HEIGHT;
                self.draw_char(byte, x, y, 0xFFFFFF, 0x000000); // 白文字、黒背景

                self.column += 1;
            }
        }
    }

    /// 文字列を書き込み
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0x20), // 非対応文字はスペース
            }
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
            let total_pixels = info.height * info.stride;
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

    let info = FB_INFO.get().unwrap();
    WRITER.call_once(|| Mutex::new(Writer::new(info)));

    // 画面をクリア
    WRITER.get().unwrap().lock().clear_screen();
}

/// フレームバッファに文字列を出力
pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    if let Some(writer) = WRITER.get() {
        x86_64::instructions::interrupts::without_interrupts(|| {
            writer.lock().write_fmt(args).unwrap();
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
