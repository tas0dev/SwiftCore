/// パニックハンドラ
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("!!! KERNEL PANIC !!!");
    log::error!("{}", info);
    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
