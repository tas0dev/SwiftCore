#![no_std]
#![no_main]

extern crate alloc;

use core::fmt::{self};
use core::mem::size_of;

use swiftlib::io;
use swiftlib::ipc;

const MAX_FILES: usize = 4;
const FILE_SIZE: usize = 512;
const MAX_HANDLES: usize = 16;

#[derive(Clone, Copy)]
struct VirtualFile {
    used: bool,
    name: [u8; 32],
    name_len: usize,
    data: [u8; FILE_SIZE],
    size: usize,
}

impl VirtualFile {
    const fn new() -> Self {
        Self { used: false, name: [0; 32], name_len: 0, data: [0; FILE_SIZE], size: 0 }
    }
}

#[derive(Clone, Copy)]
struct FileHandle {
    used: bool,
    file_idx: usize,
    offset: usize,
}

impl FileHandle {
    const fn new() -> Self {
        Self { used: false, file_idx: 0, offset: 0 }
    }
}

static mut FILES: [VirtualFile; MAX_FILES] = [VirtualFile::new(); MAX_FILES];
static mut HANDLES: [FileHandle; MAX_HANDLES] = [FileHandle::new(); MAX_HANDLES];

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct FsRequest {
    op: u64,
    arg1: u64,
    arg2: u64,
    path: [u8; 128],
}

impl FsRequest {
    const OP_OPEN: u64 = 1;
    const OP_READ: u64 = 2;
    const OP_WRITE: u64 = 3;
    const OP_CLOSE: u64 = 4;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct FsResponse {
    status: i64,
    len: u64,
    data: [u8; 128],
}

#[repr(align(8))]
struct AlignedBuffer([u8; 256]);

// 簡易的な標準出力ライター
struct Stdout;
impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        io::write_stdout(s.as_bytes());
        Ok(())
    }
}

macro_rules! print {
    ($($arg:tt)*) => ({
        let _ = core::fmt::Write::write_fmt(&mut Stdout, format_args!($($arg)*));
    });
}

macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}

#[no_mangle]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    println!("[FS] Service Started (libc version).");

    // 初期ファイル作成
    unsafe {
        FILES[0].used = true;
        let name = "readme.txt";
        for (i, b) in name.bytes().enumerate() {
            if i < 32 { FILES[0].name[i] = b; }
        }
        FILES[0].name_len = name.len();

        let content = "Welcome to SwiftCore OS!\nThis file is served by libc-based fs.service.\n";
        for (i, b) in content.bytes().enumerate() {
            if i < FILE_SIZE {
                FILES[0].data[i] = b;
            }
        }
        FILES[0].size = content.len();
    }

    println!("[FS] RamFS Initialized.");

    let mut recv_buf = AlignedBuffer([0u8; 256]);

    loop {
        let (sender, len) = ipc::ipc_recv(&mut recv_buf.0);

        if sender != 0 && (len as usize) >= size_of::<FsRequest>() {
            let req: FsRequest = unsafe { core::ptr::read(recv_buf.0.as_ptr() as *const _) };
            println!("[FS] REQ op={} from PID={}", req.op, sender);

            let mut resp = FsResponse { status: -1, len: 0, data: [0; 128] };

            match req.op {
                FsRequest::OP_OPEN => {
                    let mut found_file_idx: i64 = -1;
                    unsafe {
                        for i in 0..MAX_FILES {
                            if FILES[i].used {
                                let name_len = FILES[i].name_len;
                                let mut path_len = 0;
                                while path_len < 128 && req.path[path_len] != 0 {
                                    path_len += 1;
                                }

                                if name_len == path_len {
                                    let mut matched = true;
                                    for k in 0..name_len {
                                        if FILES[i].name[k] != req.path[k] {
                                            matched = false;
                                            break;
                                        }
                                    }
                                    if matched {
                                        found_file_idx = i as i64;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if found_file_idx != -1 {
                        let mut handle_idx: i64 = -1;
                        unsafe {
                            // 空きハンドルを探す
                            for i in 0..MAX_HANDLES {
                                if !HANDLES[i].used {
                                    HANDLES[i].used = true;
                                    HANDLES[i].file_idx = found_file_idx as usize;
                                    HANDLES[i].offset = 0;
                                    handle_idx = i as i64;
                                    break;
                                }
                            }
                        }
                        resp.status = handle_idx;
                    } else {
                         resp.status = -2; // ENOENT
                    }
                },
                FsRequest::OP_READ => {
                     let fd = req.arg1 as usize;
                     let read_len = req.arg2 as usize;

                     if fd < MAX_HANDLES && unsafe { HANDLES[fd].used } {
                         let handle = unsafe { &mut HANDLES[fd] };
                         let file_idx = handle.file_idx;

                         if file_idx < MAX_FILES && unsafe { FILES[file_idx].used } {
                            let file_size = unsafe { FILES[file_idx].size };
                            let current_offset = handle.offset;

                            if current_offset >= file_size {
                                resp.len = 0;
                                resp.status = 0; // EOF
                            } else {
                                let mut actual_len = if read_len < 128 { read_len } else { 128 };
                                if current_offset + actual_len > file_size {
                                    actual_len = file_size - current_offset;
                                }
                                unsafe {
                                    for i in 0..actual_len {
                                        resp.data[i] = FILES[file_idx].data[current_offset + i];
                                    }
                                }
                                handle.offset += actual_len;
                                resp.len = actual_len as u64;
                                resp.status = actual_len as i64;
                            }
                         } else {
                             resp.status = -9; // EBADF
                         }
                     } else {
                         resp.status = -9;
                     }
                },
                FsRequest::OP_WRITE => {
                     // 簡易実装（一旦省略）
                     resp.status = 0;
                },
                FsRequest::OP_CLOSE => {
                    let fd = req.arg1 as usize;
                    if fd < MAX_HANDLES && unsafe { HANDLES[fd].used } {
                        unsafe { HANDLES[fd].used = false; }
                        resp.status = 0;
                    } else {
                        resp.status = -9;
                    }
                },
                _ => {
                    println!("[FS] Unknown OP: {}", req.op);
                }
            }

            let resp_slice = unsafe {
                core::slice::from_raw_parts(&resp as *const _ as *const u8, size_of::<FsResponse>())
            };

            let _ = ipc::ipc_send(sender, resp_slice);

        }
    }
}