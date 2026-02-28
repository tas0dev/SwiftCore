use crate::syscall::ENODATA;

/// キーボード1文字読み取り（簡易実装）
/// 実機ドライバがない場合は常に ENODATA を返す
pub fn read_char() -> u64 {
    ENODATA
}
