use crate::sys::{syscall0, syscall1, SyscallNumber};

pub fn sleep(ms: u64) {
    unsafe {
        syscall1(SyscallNumber::Sleep as u64, ms);
    }
}

pub fn yield_now() {
    unsafe {
        syscall0(SyscallNumber::Yield as u64);
    }
}

pub fn id() -> u64 {
    unsafe {
        syscall0(SyscallNumber::GetTid as u64)
    }
}

