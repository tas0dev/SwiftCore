//! プロセス管理関連のシステムコール

use crate::task::exit_current_task;

/// Exitシステムコール
///
/// プロセスを終了する
///
/// # 引数
/// - `exit_code`: 終了コード
///
/// # 戻り値
/// このシステムコールは戻らない（プロセスが終了する）
pub fn exit(exit_code: u64) -> ! {
    crate::sprintln!("Process exiting with code: {}", exit_code);

    // スケジューラから現在のタスクを削除して終了
    exit_current_task(exit_code)
}
