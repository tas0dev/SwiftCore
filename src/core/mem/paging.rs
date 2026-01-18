//! ページング管理モジュール
//!
//! 仮想メモリとページテーブル管理

use crate::error::{KernelError, MemoryError, Result};
use crate::sprintln;
use spin::Mutex;
use x86_64::{
    structures::paging::{
        Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

static PAGE_TABLE: Mutex<Option<OffsetPageTable<'static>>> = Mutex::new(None);

/// ページングシステムを初期化
pub fn init(physical_memory_offset: u64) {
    sprintln!("Initializing paging...");

    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        let page_table = OffsetPageTable::new(level_4_table, VirtAddr::new(physical_memory_offset));
        *PAGE_TABLE.lock() = Some(page_table);
    }

    sprintln!("Paging initialized");
}

/// アクティブなレベル4ページテーブルへの参照を取得
unsafe fn active_level_4_table(physical_memory_offset: u64) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = VirtAddr::new(phys.as_u64() + physical_memory_offset);
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

/// ページをマップ
pub fn map_page(page: Page, frame: PhysFrame, flags: PageTableFlags) -> Result<()> {
    let mut page_table_lock = PAGE_TABLE.lock();
    let page_table = page_table_lock
        .as_mut()
        .ok_or(KernelError::Memory(MemoryError::NotMapped))?;

    let mut allocator_lock = super::frame::FRAME_ALLOCATOR.lock();
    let allocator = allocator_lock
        .as_mut()
        .ok_or(KernelError::Memory(MemoryError::OutOfMemory))?;

    unsafe {
        page_table
            .map_to(page, frame, flags, allocator)
            .map_err(|_| KernelError::Memory(MemoryError::InvalidAddress))?
            .flush();
    }

    Ok(())
}

/// 仮想アドレスを物理アドレスに変換
pub fn translate_addr(addr: VirtAddr) -> Option<PhysAddr> {
    use x86_64::structures::paging::mapper::Translate;

    let page_table = PAGE_TABLE.lock();
    page_table.as_ref()?.translate_addr(addr)
}

/// 物理メモリ領域を仮想アドレスへマップ
pub fn map_region(
    phys_start: u64,
    size: usize,
    virt_offset: u64,
    flags: PageTableFlags,
) -> Result<()> {
    let start = PhysAddr::new(phys_start);
    let end = PhysAddr::new(phys_start + size as u64);

    let mut current = start;
    while current < end {
        let phys_frame = PhysFrame::containing_address(current);
        let virt_addr = VirtAddr::new(current.as_u64() + virt_offset);

        if translate_addr(virt_addr).is_none() {
            let page = Page::<Size4KiB>::containing_address(virt_addr);
            map_page(page, phys_frame, flags)?;
        }

        current = PhysAddr::new(current.as_u64() + Size4KiB::SIZE);
    }

    Ok(())
}
