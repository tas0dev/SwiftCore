//! メモリ管理モジュール
//!
//! GDT、TSS、ページング、フレームアロケータ

use crate::{interrupt, sprintln, MemoryRegion, Result};

pub mod frame;
pub mod gdt;
pub mod paging;
pub mod tss;

pub fn init(physical_memory_offset: u64) {
    sprintln!("Initializing memory...");

    // 割り込みを確実に無効化
    x86_64::instructions::interrupts::disable();

    paging::init(physical_memory_offset);
    gdt::init();
    interrupt::init_idt();

    // PITを停止してからPICを初期化
    interrupt::disable_pit();
    interrupt::init_pic();

    sprintln!("Memory initialized");
}

/// メモリマップを設定してフレームアロケータを初期化
pub fn init_frame_allocator(memory_map: &'static [MemoryRegion]) -> Result<()> {
    frame::init(memory_map);

    if let Some((total, frames)) = frame::get_memory_info() {
        sprintln!(
            "Physical memory: {} MB ({} frames)",
            total / 1024 / 1024,
            frames
        );
    }

    Ok(())
}
