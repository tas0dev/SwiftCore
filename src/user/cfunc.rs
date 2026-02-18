use core::ffi::c_char;
use crate::port as rust_port;

#[allow(dead_code)]
extern "C" {
    pub fn printf(format: *const c_char, _: ...) -> i32;
    pub fn malloc(size: usize) -> *mut u8;
    pub fn free(ptr: *mut u8);
    pub fn realloc(ptr: *mut u8, size: usize) -> *mut u8;
    pub fn memalign(alignment: usize, size: usize) -> *mut u8;
}

#[no_mangle]
pub extern "C" fn inb(port: u16) -> u8 {
    rust_port::inb(port)
}

#[no_mangle]
pub extern "C" fn outb(port: u16, value: u8) {
    rust_port::outb(port, value)
}

#[no_mangle]
pub extern "C" fn inw(port: u16) -> u16 {
    rust_port::inw(port)
}

#[no_mangle]
pub extern "C" fn outw(port: u16, value: u16) {
    rust_port::outw(port, value)
}

#[no_mangle]
pub extern "C" fn inl(port: u16) -> u32 {
    rust_port::inl(port)
}

#[no_mangle]
pub extern "C" fn outl(port: u16, value: u32) {
    rust_port::outl(port, value)
}
