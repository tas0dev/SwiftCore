#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

const SYS_CONSOLE_WRITE: u64 = 5;
const SYS_EXIT: u64 = 7;
const SYS_KEYBOARD_READ: u64 = 8;
const ENODATA: u64 = u64::MAX - 4;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    write_str("SwiftCore keyboard service\n");

    loop {
        let ch = syscall0(SYS_KEYBOARD_READ);
        if ch == ENODATA {
            unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)); }
            continue;
        }

        let byte = ch as u8;
        let buf = [byte];
        let _ = syscall2(SYS_CONSOLE_WRITE, buf.as_ptr() as u64, 1);
    }
}

fn write_str(s: &str) {
    let _ = syscall2(SYS_CONSOLE_WRITE, s.as_ptr() as u64, s.len() as u64);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    write_str("keyboard service panic\n");
    let _ = syscall1(SYS_EXIT, 1);
    loop {
        unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)); }
    }
}

#[inline(always)]
fn syscall1(num: u64, arg0: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") num => ret,
            in("rdi") arg0,
            options(nostack, preserves_flags)
        );
    }
    ret
}

#[inline(always)]
fn syscall2(num: u64, arg0: u64, arg1: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") num => ret,
            in("rdi") arg0,
            in("rsi") arg1,
            options(nostack, preserves_flags)
        );
    }
    ret
}

#[inline(always)]
fn syscall0(num: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") num => ret,
            options(nostack, preserves_flags)
        );
    }
    ret
}
