//! カーネルエントリーポイント

use crate::{
    debug, info, interrupt, mem, util, vprintln, BootInfo, KernelError, MemoryRegion, Result,
};

/// カーネルエントリーポイント
#[no_mangle]
pub extern "C" fn kernel_entry(boot_info: &'static BootInfo) -> ! {
    util::console::init();

    // フレームバッファ初期化
    util::vga::init(
        boot_info.framebuffer_addr,
        boot_info.screen_width,
        boot_info.screen_height,
        boot_info.stride,
    );

    vprintln!("SwiftCore v0.1.0");
    vprintln!("Framebuffer: {:#x}", boot_info.framebuffer_addr);
    vprintln!(
        "Resolution: {}x{}",
        boot_info.screen_width,
        boot_info.screen_height
    );
    vprintln!("");

    info!(
        "Physical memory offset: {:#x}",
        boot_info.physical_memory_offset
    );

    // メモリマップを取得
    let memory_map = unsafe {
        core::slice::from_raw_parts(
            boot_info.memory_map_addr as *const MemoryRegion,
            boot_info.memory_map_len,
        )
    };

    info!("Memory map entries: {}", boot_info.memory_map_len);
    for (i, region) in memory_map.iter().enumerate() {
        debug!(
            "  Region {}: {:#x} - {:#x} ({:?})",
            i,
            region.start,
            region.start + region.len,
            region.region_type
        );
    }

    // カーネル初期化を実行
    match kernel_main(boot_info, memory_map) {
        Ok(_) => {
            // 正常に完了（通常は到達しない）
            info!("Kernel shutdown gracefully");
            halt_forever();
        }
        Err(e) => {
            // エラー時の処理
            handle_kernel_error(e);
            halt_forever();
        }
    }
}

/// カーネルメイン処理
fn kernel_main(boot_info: &'static BootInfo, memory_map: &'static [MemoryRegion]) -> Result<()> {
    info!("Initializing kernel...");

    // メモリ管理初期化
    mem::init(boot_info.physical_memory_offset);
    mem::init_frame_allocator(memory_map)?;

    info!("Kernel ready");

    // 割込みを有効化
    debug!("Enabling interrupts...");
    unsafe {
        x86_64::instructions::interrupts::enable();
    }

    // タイマー割り込みを設定（10ms周期）
    interrupt::init_pit();
    interrupt::enable_timer_interrupt();

    info!("Timer interrupt configured (10ms period)");

    // 無限ループ（永遠に実行）
    info!("Entering idle loop");
    loop {
        x86_64::instructions::hlt();
    }
}

/// カーネルエラーを処理
fn handle_kernel_error(error: KernelError) {
    use crate::error::*;

    crate::warn!("KERNEL ERROR: {}", error);
    debug!("Is fatal: {}", error.is_fatal());
    debug!("Is retryable: {}", error.is_retryable());

    match error {
        KernelError::Memory(mem_err) => {
            crate::error!("Memory error: {:?}", mem_err);
        }
        KernelError::Process(proc_err) => {
            crate::error!("Process error: {:?}", proc_err);
        }
        KernelError::Device(dev_err) => {
            crate::error!("Device error: {:?}", dev_err);
        }
        _ => {
            crate::error!("Unknown error: {:?}", error);
        }
    }

    info!("System halted.");
}

/// システムを無限ループで停止
fn halt_forever() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
