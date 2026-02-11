#![no_std]

//! SwiftCoreユーザーランドライブラリ

// TODO: いずれRustのstdを使えるようにする！あとRustツールチェーンにも組み込んでもらえるように頑張る

use core::arch::asm;

/// システムコール番号
#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallNumber {
    Yield = 1,
    GetTicks = 2,
    IpcSend = 3,
    IpcRecv = 4,
    Exec = 5,
    Exit = 6,
    Write = 7,
    Read = 8,
    GetPid = 9,
    GetTid = 10,
    Sleep = 11,
    Open = 12,
    Close = 13,
    Fork = 14,
    Wait = 15,
    Brk = 16,
    Lseek = 17,
    Fstat = 18,
    FindProcessByName = 19,
    Log = 20,
    PortIn = 21,
    PortOut = 22,
}

/// システムコールを呼び出す（引数0個）
#[inline(always)]
pub fn syscall0(num: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") num => ret,
            options(nostack, preserves_flags)
        );
    }
    ret
}

/// システムコールを呼び出す（引数1個）
#[inline(always)]
pub fn syscall1(num: u64, arg0: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") num => ret,
            in("rdi") arg0,
            options(nostack, preserves_flags)
        );
    }
    ret
}

/// システムコールを呼び出す（引数2個）
#[inline(always)]
pub fn syscall2(num: u64, arg0: u64, arg1: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") num => ret,
            in("rdi") arg0,
            in("rsi") arg1,
            options(nostack, preserves_flags)
        );
    }
    ret
}

/// システムコールを呼び出す（引数3個）
#[inline(always)]
pub fn syscall3(num: u64, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    let ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") num => ret,
            in("rdi") arg0,
            in("rsi") arg1,
            in("rdx") arg2,
            options(nostack, preserves_flags)
        );
    }
    ret
}

/// CPUを他のタスクに譲る
#[inline]
pub fn yield_now() {
    syscall0(SyscallNumber::Yield as u64);
}

/// タイマーティック数を取得
#[inline]
pub fn get_ticks() -> u64 {
    syscall0(SyscallNumber::GetTicks as u64)
}

/// プロセスを終了
#[inline]
pub fn exit(code: u64) -> ! {
    syscall1(SyscallNumber::Exit as u64, code);
    loop {
        unsafe { asm!("hlt") }
    }
}

/// ファイルディスクリプタに書き込む
#[inline]
pub fn write(fd: u64, buf: &[u8]) -> u64 {
    syscall3(
        SyscallNumber::Write as u64,
        fd,
        buf.as_ptr() as u64,
        buf.len() as u64,
    )
}

/// ファイルディスクリプタから読み込む
#[inline]
pub fn read(fd: u64, buf: &mut [u8]) -> u64 {
    syscall3(
        SyscallNumber::Read as u64,
        fd,
        buf.as_mut_ptr() as u64,
        buf.len() as u64,
    )
}

/// 標準出力に文字列を出力
#[inline]
pub fn print(s: &str) -> u64 {
    write(1, s.as_bytes())
}

/// 現在のプロセスIDを取得
#[inline]
pub fn getpid() -> u64 {
    syscall0(SyscallNumber::GetPid as u64)
}

/// 現在のスレッドIDを取得
#[inline]
pub fn gettid() -> u64 {
    syscall0(SyscallNumber::GetTid as u64)
}

/// プロセス名からPIDを検索
#[inline]
pub fn find_process_by_name(name: &str) -> u64 {
    syscall2(
        SyscallNumber::FindProcessByName as u64, 
        name.as_ptr() as u64, 
        name.len() as u64
    )
}

/// 指定されたミリ秒数の間スリープする
#[inline]
pub fn sleep(milliseconds: u64) {
    syscall1(SyscallNumber::Sleep as u64, milliseconds);
}

/// ファイルを開く（未実装）
#[inline]
pub fn open(path: &str, flags: u64) -> i64 {
    let ret = syscall2(SyscallNumber::Open as u64, path.as_ptr() as u64, flags);
    if ret == u64::MAX {
        -1
    } else {
        ret as i64
    }
}

/// ファイルを閉じる（未実装）
#[inline]
pub fn close(fd: u64) -> i64 {
    let ret = syscall1(SyscallNumber::Close as u64, fd);
    if ret == u64::MAX {
        -1
    } else {
        ret as i64
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FsRequest {
    pub op: u64,
    pub arg1: u64,
    pub arg2: u64,
    pub path: [u8; 128],
}

impl FsRequest {
    pub const OP_OPEN: u64 = 1;
    pub const OP_READ: u64 = 2;
    pub const OP_WRITE: u64 = 3;
    pub const OP_CLOSE: u64 = 4;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FsResponse {
    pub status: i64,
    pub len: u64,
    pub data: [u8; 128],
}

/// IPCメッセージ送信 (新しいAPIに合わせて修正)
#[inline]
pub fn ipc_send(dest_pid: u64, data: &[u8]) -> u64 {
    syscall3(
        SyscallNumber::IpcSend as u64,
        dest_pid,
        data.as_ptr() as u64,
        data.len() as u64,
    )
}

/// IPCメッセージ受信
/// 引数: 受信バッファ
/// 戻り値: (送信元PID, 受信サイズ)。メッセージがない場合は (0, 0) を返す。
#[inline]
pub fn ipc_recv(buf: &mut [u8]) -> (u64, usize) {
    let ret = syscall2(
        SyscallNumber::IpcRecv as u64,
        buf.as_mut_ptr() as u64,
        buf.len() as u64,
    );
    
    // エラーの場合
    if ret >= 0xffffffffffffff00 { 
        return (0, 0); 
    }
    
    let sender = ret >> 32;
    let len = (ret & 0xffffffff) as usize;
    (sender, len)
}

/// カーネルにログを書き込む
/// # 引数
/// - `msg_ptr`: ログメッセージ
/// - `len`: メッセージの長さ
/// - `level`: ログレベル（0=ERROR、1=WARNING、2=INFO、3=DEBUG）
#[inline]
pub fn log(msg_ptr: &str, len: u64, level: u64) -> u64 {
    syscall3(SyscallNumber::Log as u64, msg_ptr.as_ptr() as u64, len, level)
}