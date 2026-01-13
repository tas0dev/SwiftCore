#![no_std]
#![no_main]

extern crate alloc;

use swiftcore::{kmain, BootInfo};
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;

#[global_allocator]
static ALLOCATOR: uefi::allocator::Allocator = uefi::allocator::Allocator;

static mut BOOT_INFO: BootInfo = BootInfo {
    physical_memory_offset: 0,
    framebuffer_addr: 0,
    framebuffer_size: 0,
    screen_width: 0,
    screen_height: 0,
    stride: 0,
};

/// UEFIエントリーポイント
#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi::helpers::init(&mut system_table).expect("Failed to initialize UEFI services");

    system_table
        .stdout()
        .clear()
        .expect("Failed to clear screen");
    system_table
        .stdout()
        .output_string(cstr16!("SwiftCore starting...\n"))
        .expect("Failed to write to console");

    // Graphics Output Protocolを取得
    let gop_handle = system_table
        .boot_services()
        .get_handle_for_protocol::<GraphicsOutput>()
        .expect("Failed to get GOP handle");

    let mut gop = system_table
        .boot_services()
        .open_protocol_exclusive::<GraphicsOutput>(gop_handle)
        .expect("Failed to open GOP");

    let mode_info = gop.current_mode_info();
    let mut framebuffer = gop.frame_buffer();

    unsafe {
        BOOT_INFO.physical_memory_offset = 0;
        BOOT_INFO.framebuffer_addr = framebuffer.as_mut_ptr() as u64;
        BOOT_INFO.framebuffer_size = framebuffer.size();
        BOOT_INFO.screen_width = mode_info.resolution().0;
        BOOT_INFO.screen_height = mode_info.resolution().1;
        BOOT_INFO.stride = mode_info.stride();
    }

    unsafe {
        kmain(&BOOT_INFO);
    }
}
