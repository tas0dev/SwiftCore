#![no_std]

/// システムコールの共通インターフェース
pub mod sys;
/// libc（Newlib）サポート
pub mod newlib;
/// ipc関連のシステムコール
pub mod ipc;
/// タスク関連のシステムコール
pub mod task;
/// 時間関連のシステムコール
pub mod time;
/// 入出力関連のシステムコール
pub mod io;

use core::panic::PanicInfo;

/// パニックハンドラ
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // 簡易的なパニックハンドラ
    // _writeを使ってエラー出力できればベストだが、
    // ここでは無限ループするか、exit(1)する
    // TODO: 今後改修する
    unsafe {
       // 強制終了
       let sys_exit = 6;
       core::arch::asm!(
           "int 0x80",
           in("rax") sys_exit,
           in("rdi") 1,
           options(nostack, noreturn)
       );
    }
}
