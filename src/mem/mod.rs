//! メモリ管理モジュール
//!
//! GDT、TSS、IDT、ページング、フレームアロケータ

use crate::{sprintln, MemoryRegion, Result};

pub mod frame;
pub mod gdt;
pub mod idt;
pub mod paging;
pub mod tss;

pub fn init(physical_memory_offset: u64) {
    sprintln!("Initializing memory...");

    paging::init(physical_memory_offset);
    gdt::init();
    idt::init();
    idt::init_pic();

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
