use crate::sys::{syscall1, syscall2, SyscallNumber};

pub fn open(path: &str, flags: u64) -> Result<u64, i64> {
    let ret = unsafe {
        syscall2(SyscallNumber::Open as u64, path.as_ptr() as u64, flags)
    };

    if ret > 0xffffffffffffff00 { // エラーコード領域
         Err(-(ret as i64))
    } else {
         Ok(ret)
    }
}

pub fn close(fd: u64) -> Result<(), i64> {
    let ret = unsafe {
        syscall1(SyscallNumber::Close as u64, fd)
    };
    if ret == 0 {
        Ok(())
    } else {
        Err(-(ret as i64))
    }
}

