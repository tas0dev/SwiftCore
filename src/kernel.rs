//! カーネルエントリーポイント

use crate::{mem, sprintln, util, vprintln, BootInfo};

/// カーネルエントリーポイント
#[no_mangle]
pub extern "C" fn kmain(boot_info: &'static BootInfo) -> ! {
    util::console::init();

    // フレームバッファ初期化
    util::vga::init(
        boot_info.framebuffer_addr,
        boot_info.screen_width,
        boot_info.screen_height,
        boot_info.stride,
    );

    vprintln!("=== SwiftCore Kernel v0.1.0 ===");
    vprintln!("Framebuffer: {:#x}", boot_info.framebuffer_addr);
    vprintln!(
        "Resolution: {}x{}",
        boot_info.screen_width,
        boot_info.screen_height
    );
    vprintln!("");

    sprintln!(
        "Physical memory offset: {:#x}",
        boot_info.physical_memory_offset
    );

    // メモリ管理初期化
    mem::init(boot_info.physical_memory_offset);

    sprintln!("Kernel ready");
    vprintln!("Kernel ready - entering idle loop...");

    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
