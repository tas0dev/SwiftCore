use crate::init;
use crate::syscall::{EINVAL, ENOENT};

const MAX_PATH_LEN: usize = 256;

/// initfs 読み込み (path_ptr, path_len, buf_ptr, buf_len)
pub fn read(path_ptr: u64, path_len: u64, buf_ptr: u64, buf_len: u64) -> u64 {
    if path_ptr == 0 || buf_ptr == 0 {
        return EINVAL;
    }

    let path_len = path_len as usize;
    let buf_len = buf_len as usize;

    if path_len == 0 || path_len > MAX_PATH_LEN {
        return EINVAL;
    }

    let path_bytes = unsafe { core::slice::from_raw_parts(path_ptr as *const u8, path_len) };
    let path = match core::str::from_utf8(path_bytes) {
        Ok(s) => s,
        Err(_) => return EINVAL,
    };

    let data = match init::fs::read(path) {
        Some(d) => d,
        None => return ENOENT,
    };

    if data.len() > buf_len {
        return EINVAL;
    }

    unsafe {
        core::ptr::copy_nonoverlapping(data.as_ptr(), buf_ptr as *mut u8, data.len());
    }

    data.len() as u64
}
