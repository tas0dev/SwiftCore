#![no_std]
#![no_main]

extern crate alloc;
use core::alloc::{GlobalAlloc, Layout};
use core::ffi::c_char;

// libc関数の定義
extern "C" {
    fn printf(format: *const c_char, ...) -> i32;
    fn malloc(size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
}

// グローバルアロケータ定義（malloc/freeラッパー）
struct LibcAllocator;

unsafe impl GlobalAlloc for LibcAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        malloc(layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr);
    }
}

#[global_allocator]
static ALLOCATOR: LibcAllocator = LibcAllocator;

#[no_mangle]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    unsafe {
        let msg = b"Hello from Rust Application with libc!\n\0";
        printf(msg.as_ptr() as *const c_char);
    }
    0
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        let msg = b"App Panic!\n\0";
        printf(msg.as_ptr() as *const c_char);
    }
    loop {}
}
