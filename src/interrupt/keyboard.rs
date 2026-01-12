//! キーボード割込みハンドラ（仮）

use crate::util::fifo::Fifo;

/// キーボードスキャンコード用FIFOバッファ（最大128個）
pub static KEYBOARD_FIFO: Fifo<u8, 128> = Fifo::new();

/// キーボード割込みを処理
pub fn handle_keyboard_interrupt(scancode: u8) {
    if let Err(_) = KEYBOARD_FIFO.push(scancode) {
        // バッファ満杯の場合は古いデータを破棄（ここでは無視）
    }
}

/// キーボードからスキャンコードを取得
pub fn get_scancode() -> Option<u8> {
    KEYBOARD_FIFO.pop()
}
