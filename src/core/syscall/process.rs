//! プロセス管理関連のシステムコール

use crate::task::{exit_current_task, current_thread_id};
use super::types::{SUCCESS, ENOSYS};

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

/// GetPidシステムコール
///
/// 現在のプロセスIDを取得する
///
/// # 戻り値
/// プロセスID
pub fn getpid() -> u64 {
    if let Some(tid) = current_thread_id() {
        crate::task::with_thread(tid, |thread| {
            thread.process_id().as_u64()
        }).unwrap_or(0)
    } else {
        0
    }
}

/// GetTidシステムコール
///
/// 現在のスレッドIDを取得する
///
/// # 戻り値
/// スレッドID
pub fn gettid() -> u64 {
    if let Some(tid) = current_thread_id() {
        tid.as_u64()
    } else {
        0
    }
}

/// Brkシステムコール
/// 
/// メモリのヒープ領域サイズを変更する
pub fn brk(addr: u64) -> u64 {
    // 現在のプロセスIDを取得
    let current_tid = match current_thread_id() {
        Some(tid) => tid,
        None => return ENOSYS,
    };

    // プロセスIDを取得
    let pid = match crate::task::with_thread(current_tid, |t| t.process_id()) {
        Some(pid) => pid,
        None => return ENOSYS,
    };

    let result = crate::task::with_process_mut(pid, |process| {
        // addr == 0 なら現在の位置を返す
        if addr == 0 {
             if process.heap_start() == 0 {
                 // ヒープ領域初期化（暫定）
                 let default_heap_base = 0x4000_0000;
                 process.set_heap_start(default_heap_base);
                 process.set_heap_end(default_heap_base);
             }
             return Ok(process.heap_end());
        }

        let current_brk = process.heap_end();

        // 縮小または変化なし
        if addr <= current_brk {
            // 特に何もしない
             process.set_heap_end(addr);
             return Ok(addr);
        }

        // 拡大時にページをマップする
        let start_page = (current_brk + 4095) & !4095;
        let end_page = (addr + 4095) & !4095;

        if end_page > start_page {
            let size = end_page - start_page;
            // メモリ割り当て（書き込み可能、実行不可）
            if let Err(_) = crate::mem::paging::map_and_copy_segment(
                start_page,
                0,
                size,
                &[],
                true,
                false
            ) {
                 return Err(ENOSYS);
            }
        }

        process.set_heap_end(addr);
        Ok(addr)
    });

    match result {
        Some(Ok(addr)) => addr,
        Some(Err(err)) => err,
        None => ENOSYS,
    }
}

/// Forkシステムコール
/// 
/// プロセスを複製する
pub fn fork() -> u64 {
    ENOSYS
}

/// Sleepシステムコール
///
/// 指定されたミリ秒数の間スリープする
///
/// # 引数
/// - `milliseconds`: スリープ時間（ミリ秒）
///
/// # 戻り値
/// 成功時はSUCCESS
pub fn sleep(milliseconds: u64) -> u64 {
    // TODO: 正確なタイマーベースのスリープを実装
    // 現在は単純にyieldするだけ（タイマー割り込みがあるので時間は経過する）
    // 最大でも数回yieldするだけにする
    let yield_count = (milliseconds / 10).max(1).min(100);

    for _ in 0..yield_count {
        crate::task::yield_now();
    }

    SUCCESS
}

/// Waitシステムコール
pub fn wait(_pid: u64, _status_ptr: u64) -> u64 {
    ENOSYS
}

/// FindProcessByNameシステムコール
/// 
/// プロセス名からPIDを検索する
/// 
/// # 引数
/// - `name_ptr`: プロセス名のポインタ
/// - `len`: プロセス名の長さ
/// 
/// # 戻り値
/// 見つかった場合はPID、見つからない場合は0
pub fn find_process_by_name(name_ptr: u64, len: u64) -> u64 {
    use crate::task;
    use core::str;
    
    if name_ptr == 0 || len == 0 || len > 64 {
        return 0;
    }
    
    // ユーザー空間から名前をコピー（安全のため制限付き）
    // 本来はユーザーメモリチェックが必要
    let name_slice = unsafe { core::slice::from_raw_parts(name_ptr as *const u8, len as usize) };
    let name = match str::from_utf8(name_slice) {
        Ok(s) => s,
        Err(_) => return 0,
    };
    
    // プロセスリストを検索
    // TODO: 直接タスク管理モジュールにアクセスするのはリスキーなのでロックをかける
    // taskモジュールに検索関数を追加するのが望ましい
    task::find_process_id_by_name(name).map(|pid| pid.as_u64()).unwrap_or(0)
}
