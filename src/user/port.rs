//! ポートI/O関連のシステムコール

use crate::sys::{syscall3, SyscallNumber};

/// I/Oポートから1バイト読み取り
#[inline]
pub fn inb(port: u16) -> u8 {
    syscall3(SyscallNumber::PortIn as u64, port as u64, 1, 0) as u8
}

/// I/Oポートへ1バイト書き込み
#[inline]
pub fn outb(port: u16, value: u8) {
    syscall3(SyscallNumber::PortOut as u64, port as u64, value as u64, 1);
}

/// I/Oポートから2バイト読み取り
#[inline]
pub fn inw(port: u16) -> u16 {
    syscall3(SyscallNumber::PortIn as u64, port as u64, 2, 0) as u16
}

/// I/Oポートへ2バイト書き込み
#[inline]
pub fn outw(port: u16, value: u16) {
    syscall3(SyscallNumber::PortOut as u64, port as u64, value as u64, 2);
}

/// I/Oポートから4バイト読み取り
#[inline]
pub fn inl(port: u16) -> u32 {
    syscall3(SyscallNumber::PortIn as u64, port as u64, 4, 0) as u32
}

/// I/Oポートへ4バイト書き込み
#[inline]
pub fn outl(port: u16, value: u32) {
    syscall3(SyscallNumber::PortOut as u64, port as u64, value as u64, 4);
}
