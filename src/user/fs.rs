//! ファイルシステム関連のシステムコール（ユーザー側）

use super::sys::{syscall1, syscall2, syscall3, SyscallNumber};

/// ディレクトリを作成
///
/// # 引数
/// - `path`: ディレクトリパス
/// - `mode`: パーミッション
///
/// # 戻り値
/// 成功時は0、失敗時はエラーコード
pub fn mkdir(path: &str, mode: u32) -> u64 {
    syscall2(
        SyscallNumber::Mkdir as u64,
        path.as_ptr() as u64,
        mode as u64,
    )
}

/// ディレクトリを削除
///
/// # 引数
/// - `path`: ディレクトリパス
///
/// # 戻り値
/// 成功時は0、失敗時はエラーコード
pub fn rmdir(path: &str) -> u64 {
    syscall1(
        SyscallNumber::Rmdir as u64,
        path.as_ptr() as u64,
    )
}

/// ディレクトリエントリを読み取る
///
/// # 引数
/// - `fd`: ディレクトリのファイルディスクリプタ
/// - `buf`: バッファ
///
/// # 戻り値
/// 読み取ったバイト数、またはエラーコード
pub fn readdir(fd: u64, buf: &mut [u8]) -> u64 {
    syscall3(
        SyscallNumber::Readdir as u64,
        fd,
        buf.as_mut_ptr() as u64,
        buf.len() as u64,
    )
}

/// カレントディレクトリを変更
///
/// # 引数
/// - `path`: ディレクトリパス
///
/// # 戻り値
/// 成功時は0、失敗時はエラーコード
pub fn chdir(path: &str) -> u64 {
    syscall1(
        SyscallNumber::Chdir as u64,
        path.as_ptr() as u64,
    )
}
