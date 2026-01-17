//! 起動時に実行する初期化処理をまとめたモジュール

pub mod fs;

fn kinit() {
    util::console::init();
    util::vga::init(
        boot_info.framebuffer_addr,
        boot_info.screen_width,
        boot_info.screen_height,
        boot_info.stride,
    );

    let memory_map = unsafe {
        core::slice::from_raw_parts(
            boot_info.memory_map_addr as *const MemoryRegion,
            boot_info.memory_map_len,
        )
    };

    for (i, region) in memory_map.iter().enumerate() {
        crate::debug!(
            "  Region {}: {:#x} - {:#x} ({:?})",
            i,
            region.start,
            region.start + region.len,
            region.region_type
        );
    }

    task::init_scheduler();

    mem::init(boot_info.physical_memory_offset);
    mem::init_frame_allocator(memory_map)?;

    fs::init();
    
    unsafe {
        x86_64::instructions::interrupts::enable();
    }

    interrupt::init_pit();
    interrupt::enable_timer_interrupt();
}