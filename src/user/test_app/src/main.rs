#![no_std]
#![no_main]

extern crate test_app;
use core::panic::PanicInfo;

use test_app::{yield_now, print, exit};

/// ユーザーアプリのエントリーポイント
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Writeシステムコールのテスト
    print("Hello from user space!\n");
    print("Testing write syscall...\n");

    let mut counter = 0u64;

    loop {
        // カウンターをインクリメント
        counter = counter.wrapping_add(1);

        // 10000回ごとにメッセージを出力
        if counter % 10000 == 0 {
            print("User app is running...\n");
            yield_now();
        }

        // 50000回でループを抜ける（テスト用）
        if counter >= 50000 {
            break;
        }
    }

    // 終了メッセージ
    print("User app finished. Exiting...\n");
    
    // exitシステムコールでプロセスを終了
    exit(0);
}

/// パニックハンドラ
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print("PANIC in user space!\n");
    loop {
        yield_now();
    }
}
