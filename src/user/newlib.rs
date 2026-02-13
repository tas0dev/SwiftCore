//! Newlib サポート用のシステムコールグルーコード

use super::sys::{syscall1, syscall2, syscall3, SyscallNumber};

#[no_mangle]
pub extern "C" fn _write(fd: i32, buf: *const u8, len: usize) -> isize {
    syscall3(SyscallNumber::Write as u64, fd as u64, buf as u64, len as u64) as isize
}

#[no_mangle]
pub extern "C" fn _read(fd: i32, buf: *mut u8, len: usize) -> isize {
    syscall3(SyscallNumber::Read as u64, fd as u64, buf as u64, len as u64) as isize
}

#[no_mangle]
pub extern "C" fn _close(fd: i32) -> i32 {
    syscall1(SyscallNumber::Close as u64, fd as u64) as i32
}

#[no_mangle]
pub extern "C" fn _lseek(fd: i32, offset: isize, whence: i32) -> isize {
    syscall3(SyscallNumber::Lseek as u64, fd as u64, offset as u64, whence as u64) as isize
}

#[no_mangle]
pub extern "C" fn _exit(code: i32) -> ! {
    syscall1(SyscallNumber::Exit as u64, code as u64);
    loop {}
}

#[no_mangle]
pub extern "C" fn _fstat(fd: i32, stat: *mut u8) -> i32 {
    syscall2(SyscallNumber::Fstat as u64, fd as u64, stat as u64) as i32
}

#[no_mangle]
pub extern "C" fn _isatty(fd: i32) -> i32 {
    // 簡易実装: 標準入出力(0,1,2)はTTYとみなす
    if fd >= 0 && fd <= 2 {
        1
    } else {
        0
    }
}

// ヒープの現在の末端アドレス
static mut HEAP_END: usize = 0;

#[no_mangle]
pub extern "C" fn _sbrk(incr: isize) -> *mut u8 {
    unsafe {
        // ヒープの初期化が行われていない場合
        if HEAP_END == 0 {
            // 現在のbrk（ヒープの末端）を取得するために brk(0) を呼ぶ
            let cur = syscall1(SyscallNumber::Brk as u64, 0);

            // エラー判定 (0や異常値が返ってきた場合)
            if cur == 0 || cur > 0xffff_ffff_ffff_f000 {
                return -1_isize as *mut u8;
            }
            HEAP_END = cur as usize;
        }

        let old_heap_end = HEAP_END;

        // sbrk(0) の場合は現在値を返すだけ
        if incr == 0 {
            return old_heap_end as *mut u8;
        }

        let new_heap_end = (old_heap_end as isize + incr) as usize;
        let ret = syscall1(SyscallNumber::Brk as u64, new_heap_end as u64);

        if ret == new_heap_end as u64 {
            HEAP_END = new_heap_end;
            old_heap_end as *mut u8
        } else {
            -1_isize as *mut u8
        }
    }
}

#[no_mangle]
pub extern "C" fn _getpid() -> i32 {
    syscall1(SyscallNumber::GetPid as u64, 0) as i32
}

#[no_mangle]
pub extern "C" fn _kill(pid: i32, sig: i32) -> i32 {
    // TODO: 実装する
    -1
}

