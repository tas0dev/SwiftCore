//! カーネルエントリーポイント

use crate::{interrupt, mem, task, util, BootInfo, KernelError, MemoryRegion, Result};

/// カーネルエントリーポイント
#[no_mangle]
pub extern "C" fn kernel_entry(boot_info: &'static BootInfo) -> ! {
    util::log::set_level(util::log::LogLevel::Info);
    init::kinit();

    match kernel_main(boot_info, memory_map) {
        Ok(_) => {
            crate::info!("Kernel shutdown gracefully");
            halt_forever();
        }
        Err(e) => {
            crate::error::handle_kernel_error(e);
            halt_forever();
        }
    }
}

/// カーネルメイン処理
fn kernel_main(boot_info: &'static BootInfo, memory_map: &'static [MemoryRegion]) -> Result<()> {
    crate::info!("Initializing kernel...");
    crate::info!("Memory map entries: {}", boot_info.memory_map_len);

    crate::vprintln!("Framebuffer: {:#x}", boot_info.framebuffer_addr);
    crate::vprintln!(
        "Resolution: {}x{}",
        boot_info.screen_width,
        boot_info.screen_height
    );

    crate::info!(
        "Physical memory offset: {:#x}",
        boot_info.physical_memory_offset
    );

    crate::info!("Starting task scheduler...");
    task::start_scheduling();

    #[allow(unreachable_code)]
    Ok(())
}

/// システムを無限ループで停止
fn halt_forever() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
