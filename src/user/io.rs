//! I/O関連のシステムコールラッパー

use crate::sys::{syscall1, syscall3, SyscallNumber};

/// 標準出力のファイルディスクリプタ
pub const STDOUT: u64 = 1;
/// 標準エラー出力のファイルディスクリプタ
pub const STDERR: u64 = 2;
/// 標準入力のファイルディスクリプタ
pub const STDIN: u64 = 0;

/// ファイルディスクリプタに書き込む
///
/// # 引数
/// - `fd`: ファイルディスクリプタ
/// - `buf`: 書き込むデータ
///
/// # 戻り値
/// 書き込んだバイト数、またはエラーコード
#[inline]
pub fn write(fd: u64, buf: &[u8]) -> u64 {
    syscall3(
        SyscallNumber::Write as u64,
        fd,
        buf.as_ptr() as u64,
        buf.len() as u64,
    )
}

/// 標準出力に書き込む
///
/// # 引数
/// - `buf`: 書き込むデータ
///
/// # 戻り値
/// 書き込んだバイト数、またはエラーコード
#[inline]
pub fn write_stdout(buf: &[u8]) -> u64 {
    write(STDOUT, buf)
}

/// 標準エラー出力に書き込む
///
/// # 引数
/// - `buf`: 書き込むデータ
///
/// # 戻り値
/// 書き込んだバイト数、またはエラーコード
#[inline]
pub fn write_stderr(buf: &[u8]) -> u64 {
    write(STDERR, buf)
}

/// 標準出力に文字列を書き込む
///
/// # 引数
/// - `s`: 書き込む文字列
///
/// # 戻り値
/// 書き込んだバイト数、またはエラーコード
#[inline]
pub fn print(s: &str) -> u64 {
    write_stdout(s.as_bytes())
}

/// ファイルディスクリプタから読み込む
///
/// # 引数
/// - `fd`: ファイルディスクリプタ
/// - `buf`: 読み込むバッファ
///
/// # 戻り値
/// 読み込んだバイト数、またはエラーコード
#[inline]
pub fn read(fd: u64, buf: &mut [u8]) -> u64 {
    syscall3(
        SyscallNumber::Read as u64,
        fd,
        buf.as_mut_ptr() as u64,
        buf.len() as u64,
    )
}

/// プロセスを終了する
///
/// # 引数
/// - `code`: 終了コード
///
/// # 戻り値
/// この関数は戻らない...そう、多分。いや、戻られたら困るんだけど。
#[inline]
pub fn exit(code: u64) -> ! {
    syscall1(SyscallNumber::Exit as u64, code);
    // システムコールが戻らない場合に備えて
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
