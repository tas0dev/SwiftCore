#![no_std]
#![no_main]

extern crate fs_service;
use core::panic::PanicInfo;
use fs_service::{print, print_u64, ipc_recv, ipc_send, yield_now, FsRequest, FsResponse};

// --- RamFS Definitions ---
const MAX_FILES: usize = 4;
const FILE_SIZE: usize = 512;

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

static mut FILES: [VirtualFile; MAX_FILES] = [VirtualFile::new(); MAX_FILES];

#[repr(align(8))]
struct AlignedBuffer([u8; 256]);

/// FS Service Entry Point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    print("[FS] Service Started.\n");

    // 初期ファイル作成
    unsafe {
        FILES[0].used = true;
        let name = "readme.txt";
        // 文字列コピー
        for (i, b) in name.bytes().enumerate() {
            if i < 32 { FILES[0].name[i] = b; }
        }
        FILES[0].name_len = name.len();

        let content = "Welcome to SwiftCore OS!\nThis file is served by fs.service from RamFS.\n";
        for (i, b) in content.bytes().enumerate() {
            if i < FILE_SIZE {
                FILES[0].data[i] = b;
            }
        }
        FILES[0].size = content.len();
    }

    print("[FS] RamFS Initialized. 'readme.txt' created.\n");
    print("[FS] Waiting for requests...\n");

    let mut recv_buf = AlignedBuffer([0u8; 256]);

    loop {
        let (sender, len) = ipc_recv(&mut recv_buf.0);
        if sender != 0 && len >= size_of::<FsRequest>() {
            // 受信データをリクエストとして解釈
            let req: FsRequest = unsafe { core::ptr::read(recv_buf.0.as_ptr() as *const _) };

            print("[FS] REQ op=");
            print_u64(req.op);
            print(" from PID=");
            print_u64(sender);
            print("\n");

            let mut resp = FsResponse { status: -1, len: 0, data: [0; 128] };

            match req.op {
                FsRequest::OP_OPEN => {
                    print("[FS] OP_OPEN\n");
                    let mut found_idx: i64 = -1;

                    unsafe {
                        for i in 0..MAX_FILES {
                            if FILES[i].used {
                                // 厳密な比較を行う
                                let name_len = FILES[i].name_len;

                                // リクエストのパス長を取得（null終端または最大長）
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
                                        found_idx = i as i64;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    resp.status = found_idx; // FDとしてインデックスを返す
                    if found_idx == -1 {
                        print("[FS] ERROR: File not found\n");
                    } else {
                        print("[FS] Success: FD=");
                        print_u64(found_idx as u64);
                        print("\n");
                    }
                },
                FsRequest::OP_READ => {
                     // arg1=fd, arg2=len
                     let fd = req.arg1 as usize;
                     let read_len = req.arg2 as usize;

                     if fd < MAX_FILES && unsafe { FILES[fd].used } {
                         let file_size = unsafe { FILES[fd].size };
                         let actual_len = if read_len < 128 { read_len } else { 128 }; // レスポンスバッファ制限
                         let actual_len = if actual_len < file_size { actual_len } else { file_size };

                         unsafe {
                             for i in 0..actual_len {
                                 resp.data[i] = FILES[fd].data[i];
                             }
                         }
                         resp.len = actual_len as u64;
                         resp.status = actual_len as i64;
                     } else {
                         resp.status = -9; // EBADF
                     }
                },
                _ => {
                    print("[FS] Unknown OP\n");
                }
            }

            // レスポンス送信
            let resp_slice = unsafe {
                core::slice::from_raw_parts(&resp as *const _ as *const u8, size_of::<FsResponse>())
            };
            ipc_send(sender, resp_slice);

        } else {
            yield_now();
        }
    }
}

/// パニックハンドラ
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print("PANIC in user space!\n");
    loop {
        yield_now();
    }
}
