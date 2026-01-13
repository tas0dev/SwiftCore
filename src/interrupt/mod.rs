//! 割込み管理モジュール
//!
//! IDT、PIC、割込みハンドラを管理

pub mod idt;
pub mod pic;
pub mod timer;

use crate::sprintln;

/// 割込みシステムを初期化
pub fn init() {
    sprintln!("Initializing interrupts...");

    // PICを初期化
    unsafe {
        pic::PICS.lock().initialize();
    }
    sprintln!("PIC initialized");

    // IDTを初期化
    idt::init();

    // タイマーとキーボード割込みを有効化
    unsafe {
        pic::PICS.lock().enable_timer_and_keyboard();
    }
    sprintln!("Timer and keyboard interrupts unmasked");

    // CPU割込みを有効化
    x86_64::instructions::interrupts::enable();

    sprintln!("Interrupts enabled");
}
