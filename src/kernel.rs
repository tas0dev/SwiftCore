#![no_std]
#![feature(abi_x86_interrupt)]

pub mod mem;
pub mod panic;
pub mod util;

#[repr(C)]
pub struct BootInfo {
    /// 物理メモリオフセット
    pub physical_memory_offset: u64,
}

/// カーネルエントリーポイント
#[no_mangle]
pub extern "C" fn kmain(boot_info: &'static BootInfo) -> ! {
    // シリアルポートを初期化
    util::serial::init();
    println!("=== SwiftCore Kernel v0.1.0 ===");
    println!("Serial output initialized");
    println!(
        "Physical memory offset: {:#x}",
        boot_info.physical_memory_offset
    );

    // メモリ管理初期化
    mem::init(boot_info.physical_memory_offset);

    println!("Kernel ready");

    // 割り込みを無効にしてメインループへ
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("cli"); // 割り込み無効化
    }

    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
