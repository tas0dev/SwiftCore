//! ユーザーモード実行サポート

use core::arch::asm;
use crate::mem::gdt;

/// ユーザーモードでコードを実行する
///
/// # 引数
/// - `entry`: ユーザーモードで実行する関数のアドレス
/// - `user_stack`: ユーザースタックのトップアドレス
///
/// # 注意
/// この関数は戻らない
pub unsafe fn jump_to_usermode(entry: u64, user_stack: u64) -> ! {
    let user_cs = gdt::user_code_selector().0 as u64 | 3; // RPL=3
    let user_ss = gdt::user_data_selector().0 as u64 | 3; // RPL=3

    // iretqスタックフレームを構築:
    // SS, RSP, RFLAGS, CS, RIP
    asm!(
        "cli",
        // iretq用のスタックフレームをプッシュ
        "push {ss}",       // SS (ユーザーデータセグメント)
        "push {rsp}",      // RSP (ユーザースタック)
        "push 0x202",      // RFLAGS (IF=1, IOPL=0)
        "push {cs}",       // CS (ユーザーコードセグメント)
        "push {rip}",      // RIP (エントリーポイント)

        // ユーザーデータセグメントをセット
        "mov ax, {ss:x}",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",

        // iretqでユーザーモードへジャンプ
        "iretq",

        ss = in(reg) user_ss,
        rsp = in(reg) user_stack,
        cs = in(reg) user_cs,
        rip = in(reg) entry,
        options(noreturn)
    );
}
