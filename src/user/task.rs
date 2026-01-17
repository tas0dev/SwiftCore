//! タスク系システムコール（ユーザー側）

use super::sys::{syscall0, SyscallNumber};

/// スケジューラに実行権を譲る
pub fn yield_now() {
    let _ = syscall0(SyscallNumber::Yield as u64);
}
