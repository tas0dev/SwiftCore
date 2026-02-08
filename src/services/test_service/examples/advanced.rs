#![no_std]
#![no_main]

extern crate test_app;
use core::panic::PanicInfo;

use test_app::yield_now;

/// ユーザーアプリのエントリーポイント
///
/// このアプリケーションは以下を実行します:
/// 1. タイマーティック値を取得
/// 2. カウンターをインクリメント
/// 3. 定期的にyieldして他のタスクに譲る
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut counter = 0u64;

    loop {
        counter = counter.wrapping_add(1);
        // 1000000回でループを抜ける（テスト用）
        if counter >= 1000000 {
            break;
        }
    }

    loop {
        // TODO: write()システムコールが実装されたら結果を出力
        // write(1, "Counter: {}, Time: {} ticks\n", counter, end_ticks - start_ticks);
    }
}

/// パニックハンドラ
///
/// パニック時は単純にyieldを繰り返してCPUを消費しない
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        yield_now();
    }
}
