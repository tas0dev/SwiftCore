#![no_std]
#![no_main]

// _startシンボルを定義

use core::arch::global_asm;

global_asm!(
    ".section .text",
    ".global _start",
    "_start:",
    // カーネルによってスタックは既にアロケートされている
    "and rsp, -16",

    // argc, argv の準備
    // 現在のカーネル実装では引数は渡されていないため、argc=0, argv=NULL とする
    "xor edi, edi", // argc = 0
    "xor esi, esi", // argv = NULL

    "call main",

    // main の戻り値 (rax) を引数に exit を呼ぶ
    "mov edi, eax",
    "call _exit",
);

extern "C" {
    fn main(argc: i32, argv: *const *const u8) -> i32;
    fn _exit(code: i32) -> !;
}

