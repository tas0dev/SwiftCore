//! タスク系システムコール（ユーザー側）

use super::sys::{syscall0, SyscallNumber};

/// スケジューラに実行権を譲る
pub fn yield_now() {
    let _ = syscall0(SyscallNumber::Yield as u64);
}

/// 現在のスレッドを終了
pub fn exit(code: u64) -> u64 {
    super::sys::syscall1(SyscallNumber::Exit as u64, code)
}
