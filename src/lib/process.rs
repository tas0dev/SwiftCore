use crate::sys::{syscall0, syscall1, SyscallNumber};

pub fn exit(code: u64) -> ! {
    unsafe {
        syscall1(SyscallNumber::Exit as u64, code);
        loop {
            core::arch::asm!("hlt");
        }
    }
}

pub fn id() -> u64 {
    unsafe {
        syscall0(SyscallNumber::GetPid as u64)
    }
}

