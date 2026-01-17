//! IPC 系システムコール（ユーザー側）

use super::sys::{syscall1, syscall2, SyscallNumber};

/// IPC送信（宛先スレッドID, 値）
pub fn ipc_send(dest_thread_id: u64, value: u64) -> u64 {
    syscall2(SyscallNumber::IpcSend as u64, dest_thread_id, value)
}

/// IPC受信（送信元IDを受け取る場合はSome）
pub fn ipc_recv(sender_out: Option<&mut u64>) -> u64 {
    let ptr = sender_out
        .map(|s| s as *mut u64 as u64)
        .unwrap_or(0);
    syscall1(SyscallNumber::IpcRecv as u64, ptr)
}
