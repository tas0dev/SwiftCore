use crate::driver::ps2_keyboard;
use crate::syscall::ENODATA;

/// キーボード1文字読み取り
pub fn read_char() -> u64 {
    match ps2_keyboard::read_char() {
        Some(ch) => ch as u64,
        None => ENODATA,
    }
}
