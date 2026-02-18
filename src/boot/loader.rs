#![no_std]
#![no_main]

extern crate alloc;

use swiftcore::{kernel_entry, BootInfo, MemoryRegion, MemoryType};
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use linked_list_allocator::LockedHeap;
use core::sync::atomic::{AtomicBool, Ordering};
use core::alloc::{GlobalAlloc, Layout};

struct BootAllocator {
    kernel: LockedHeap,
    use_kernel: AtomicBool,
}

unsafe impl GlobalAlloc for BootAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if self.use_kernel.load(Ordering::Relaxed) {
             self.kernel.alloc(layout)
        } else {
             uefi::allocator::Allocator.alloc(layout)
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if self.use_kernel.load(Ordering::Relaxed) {
             self.kernel.dealloc(ptr, layout)
        } else {
             uefi::allocator::Allocator.dealloc(ptr, layout)
        }
    }
}

#[global_allocator]
static ALLOCATOR: BootAllocator = BootAllocator {
    kernel: LockedHeap::empty(),
    use_kernel: AtomicBool::new(false),
};

static mut BOOT_INFO: BootInfo = BootInfo {
    physical_memory_offset: 0,
    framebuffer_addr: 0,
    framebuffer_size: 0,
    screen_width: 0,
    screen_height: 0,
    stride: 0,
    memory_map_addr: 0,
    memory_map_len: 0,
    memory_map_entry_size: 0,
    allocator_addr: 0,
    kernel_heap_addr: 0,
};

// メモリマップを静的に保存
static mut MEMORY_MAP: [MemoryRegion; 256] = [MemoryRegion {
    start: 0,
    len: 0,
    region_type: MemoryType::Reserved,
}; 256];

/// UEFIエントリーポイント
#[entry]
unsafe fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    if let Err(_) = uefi::helpers::init(&mut system_table) {
        return Status::UNSUPPORTED;
    }

    let _ = system_table.stdout().clear();
    let _ = system_table
        .stdout()
        .output_string(cstr16!("SwiftCore starting...\n"));

    // Graphics Output Protocolを取得してフレームバッファ情報を保存
    let (fb_addr, fb_size, screen_w, screen_h, stride) = {
        let gop_handle = match system_table
            .boot_services()
            .get_handle_for_protocol::<GraphicsOutput>()
        {
            Ok(handle) => handle,
            Err(_) => return Status::UNSUPPORTED,
        };

        let mut gop = match system_table
            .boot_services()
            .open_protocol_exclusive::<GraphicsOutput>(gop_handle)
        {
            Ok(gop) => gop,
            Err(_) => return Status::UNSUPPORTED,
        };

        let mode_info = gop.current_mode_info();
        let mut framebuffer = gop.frame_buffer();

        (
            framebuffer.as_mut_ptr() as u64,
            framebuffer.size(),
            mode_info.resolution().0,
            mode_info.resolution().1,
            mode_info.stride(),
        )
    };

    // Boot Servicesを終了してメモリマップを取得
    let (_system_table, memory_map_iter) =
        unsafe { system_table.exit_boot_services(uefi::table::boot::MemoryType::LOADER_DATA) };

    // メモリマップを静的配列にコピー
    let map_count;
    unsafe {
        let mut count = 0;
        for (i, desc) in memory_map_iter.entries().enumerate() {
            if i >= 256 {
                break;
            }
            MEMORY_MAP[i] = MemoryRegion {
                start: desc.phys_start,
                len: desc.page_count * 4096,
                region_type: match desc.ty {
                    uefi::table::boot::MemoryType::CONVENTIONAL => MemoryType::Usable,
                    uefi::table::boot::MemoryType::ACPI_RECLAIM => MemoryType::AcpiReclaimable,
                    uefi::table::boot::MemoryType::ACPI_NON_VOLATILE => MemoryType::AcpiNvs,
                    uefi::table::boot::MemoryType::UNUSABLE => MemoryType::BadMemory,
                    uefi::table::boot::MemoryType::LOADER_CODE
                    | uefi::table::boot::MemoryType::LOADER_DATA => {
                        MemoryType::BootloaderReclaimable
                    }
                    _ => MemoryType::Reserved,
                },
            };
            count += 1;
        }
        map_count = count;
    }

    #[allow(static_mut_refs)]
    unsafe {
        BOOT_INFO.physical_memory_offset = 0;
        BOOT_INFO.framebuffer_addr = fb_addr;
        BOOT_INFO.framebuffer_size = fb_size;
        BOOT_INFO.screen_width = screen_w;
        BOOT_INFO.screen_height = screen_h;
        BOOT_INFO.stride = stride;
        BOOT_INFO.memory_map_addr = MEMORY_MAP.as_ptr() as u64;
        BOOT_INFO.memory_map_len = map_count;
        BOOT_INFO.memory_map_entry_size = size_of::<MemoryRegion>();
    }

    // ここでカーネルアロケータに切り替え
    unsafe {
        BOOT_INFO.allocator_addr = &ALLOCATOR.use_kernel as *const _ as u64;
        BOOT_INFO.kernel_heap_addr = &ALLOCATOR.kernel as *const _ as u64;
    }

    kernel_entry(&*&raw const BOOT_INFO);
}
