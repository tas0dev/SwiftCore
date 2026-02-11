//! システムコール定義

use core::arch::asm;

/// システムコール番号
#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallNumber {
    Yield = 1,
    GetTicks = 2,
    IpcSend = 3,
    IpcRecv = 4,
    Exec = 5,
    Exit = 6,
    Write = 7,
    Read = 8,
    GetPid = 9,
    GetTid = 10,
    Sleep = 11,
    Open = 12,
    Close = 13,
    Fork = 14,
    Wait = 15,
    Brk = 16,
    Lseek = 17,
    Fstat = 18,
    FindProcessByName = 19,
    Log = 20,
    PortIn = 21,
    PortOut = 22,
}

#[inline(always)]
pub unsafe fn syscall0(num: u64) -> u64 {
    let ret: u64;
    asm!(
        "int 0x80",
        inlateout("rax") num => ret,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall1(num: u64, arg0: u64) -> u64 {
    let ret: u64;
    asm!(
        "int 0x80",
        inlateout("rax") num => ret,
        in("rdi") arg0,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall2(num: u64, arg0: u64, arg1: u64) -> u64 {
    let ret: u64;
    asm!(
        "int 0x80",
        inlateout("rax") num => ret,
        in("rdi") arg0,
        in("rsi") arg1,
        options(nostack, preserves_flags)
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall3(num: u64, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    let ret: u64;
    asm!(
        "int 0x80",
        inlateout("rax") num => ret,
        in("rdi") arg0,
        in("rsi") arg1,
        in("rdx") arg2,
        options(nostack, preserves_flags)
    );
    ret
}

