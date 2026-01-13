#![no_std]
#![feature(abi_x86_interrupt)]
#![allow(unused)]

/// カーネル本体
pub mod kernel;

/// 割込み管理
pub mod interrupt;

/// メモリ管理、GDT、TSSを含む
pub mod mem;

/// パニックハンドラ
pub mod panic;

/// ユーティリティモジュール
pub mod util;

pub use kernel::kmain;

#[repr(C)]
pub struct BootInfo {
    /// 物理メモリオフセット
    pub physical_memory_offset: u64,
    /// フレームバッファアドレス
    pub framebuffer_addr: u64,
    /// フレームバッファサイズ
    pub framebuffer_size: usize,
    /// 画面の幅（ピクセル）
    pub screen_width: usize,
    /// 画面の高さ（ピクセル）
    pub screen_height: usize,
    /// 1行あたりのバイト数
    pub stride: usize,
}
