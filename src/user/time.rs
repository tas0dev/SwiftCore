//! 時刻系システムコール（ユーザー側）

use super::sys::{syscall0, SyscallNumber};

/// タイマーティック数を取得
pub fn get_ticks() -> u64 {
    syscall0(SyscallNumber::GetTicks as u64)
}
