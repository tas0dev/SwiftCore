#![no_std]
#![no_main]

extern crate fs_service;
use core::panic::PanicInfo;

use fs_service::{print, print_u64, ipc_recv, yield_now};

/// FS Service Entry Point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    print("[FS] Service Started.\n");
    print("[FS] Waiting for IPC messages...\n");

    loop {
        let (sender, val) = ipc_recv();
        if sender != 0 {
            print("[FS] MSG from ");
            print_u64(sender);
            print(": val=");
            print_u64(val);
            print("\n");

            // コマンド処理（仮実装）
            match val {
                1 => { print("[FS] Ping received\n"); },
                _ => { print("[FS] Unknown command\n"); },
            };
        } else {
            // メッセージがない場合はイールドする
            yield_now();
        }
    }
}

/// パニックハンドラ
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print("PANIC in user space!\n");
    loop {
        yield_now();
    }
}
