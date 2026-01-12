//! カーネルエントリーポイント

use crate::{interrupt, mem, sprintln, util, BootInfo};

/// カーネルエントリーポイント
#[no_mangle]
pub extern "C" fn kmain(boot_info: &'static BootInfo) -> ! {
    // シリアルポートを初期化
    util::serial::init();
    sprintln!("=== SwiftCore Kernel v0.1.0 ===");
    sprintln!("Serial output initialized");
    sprintln!(
        "Physical memory offset: {:#x}",
        boot_info.physical_memory_offset
    );

    // メモリ管理初期化
    mem::init(boot_info.physical_memory_offset);

    // 割込みシステム初期化
    interrupt::init();

    sprintln!("Kernel ready");
    sprintln!("Waiting for keyboard input...");
    
    // メインループ
    let mut count = 0u64;
    loop {
        count += 1;
        if count % 100000 == 0 {
            sprintln!("Loop iteration: {}", count);
        }
        
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("pause");
        }
    }
}
