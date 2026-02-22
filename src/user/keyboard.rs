//! キーボード系システムコール（ユーザー側）

use super::sys::{syscall0, SyscallNumber, ENODATA};

/// 1文字読み取り（なければ None）
pub fn read_char() -> Option<u8> {
    let ret = syscall0(SyscallNumber::KeyboardRead as u64);
    if ret == ENODATA {
        None
    } else {
        Some(ret as u8)
    }
}
