#[no_mangle]
pub unsafe extern "C" fn write(fd: i32, buf: *const u8, count: usize) -> isize {
    let slice = core::slice::from_raw_parts(buf, count);
    let ret = crate::io::write(fd as u64, slice);
    
    if ret == u64::MAX {
        -1
    } else {
        ret as isize
    }
}

#[no_mangle]
pub unsafe extern "C" fn read(fd: i32, buf: *mut u8, count: usize) -> isize {
    let slice = core::slice::from_raw_parts_mut(buf, count);
    let ret = crate::io::read(fd as u64, slice);
    
    if ret == u64::MAX {
        -1
    } else {
        ret as isize
    }
}

#[no_mangle]
pub unsafe extern "C" fn open(path: *const u8, flags: i32) -> i32 {
    // TODO: 実装する
    -1
}

#[no_mangle]
pub unsafe extern "C" fn close(fd: i32) -> i32 {
    crate::io::close(fd as u64) as i32
}

#[no_mangle]
pub unsafe extern "C" fn nanosleep(_req: *const (), _rem: *mut ()) -> i32 {
    // TODO: 実装する
    0
}