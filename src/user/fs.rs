//! initfs 系システムコール（ユーザー側）

use super::sys::{syscall4, SyscallNumber};

/// initfs から読み込み
pub fn read(path: &str, buf: &mut [u8]) -> u64 {
    syscall4(
        SyscallNumber::InitfsRead as u64,
        path.as_ptr() as u64,
        path.len() as u64,
        buf.as_mut_ptr() as u64,
        buf.len() as u64,
    )
}
