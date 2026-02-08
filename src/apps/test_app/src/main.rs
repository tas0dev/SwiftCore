#![no_std]
#![no_main]

extern crate test_app;
use core::panic::PanicInfo;

use test_app::{exit, find_process_by_name, ipc_recv, ipc_send, print, sleep, yield_now, FsRequest, FsResponse};

/// ユーザーアプリのエントリーポイント
#[no_mangle]
pub extern "C" fn _start() -> ! {
    print("TestApp Started.\n");

    // FSサービスのPIDを検索
    print("Looking for fs.service...\n");
    let mut fs_pid = 0;
    for _ in 0..5 {
        fs_pid = find_process_by_name("core.service.fs");
        if fs_pid != 0 {
            break;
        }
        sleep(100);
    }

    if fs_pid == 0 {
        print("Error: fs.service not found.\n");
        exit(1);
    }

    print("Found fs.service at PID: ");
    print_u64(fs_pid);
    print("\n");

    // ファイルを開く
    let filename = "readme.txt";
    print("Opening ");
    print(filename);
    print("...\n");

    let mut req = FsRequest {
        op: FsRequest::OP_OPEN,
        arg1: 0,
        arg2: 0,
        path: [0; 128],
    };
    
    for (i, b) in filename.bytes().enumerate() {
        if i < 128 { req.path[i] = b; }
    }
    
    let req_slice = unsafe {
        core::slice::from_raw_parts(&req as *const _ as *const u8, core::mem::size_of::<FsRequest>())
    };
    
    ipc_send(fs_pid, req_slice);
    
    // レスポンス受信
    let mut resp_buf = [0u8; 256];
    let mut fd: i64 = -1;
    
    // タイムアウト付き受信（簡易）
    for _ in 0..10 {
        let (sender, len) = ipc_recv(&mut resp_buf);
        if sender == fs_pid && len >= core::mem::size_of::<FsResponse>() {
            let resp: FsResponse = unsafe { core::ptr::read(resp_buf.as_ptr() as *const _) };
            fd = resp.status;
            break;
        }
        yield_now();
    }
    
    if fd < 0 {
        print("Failed to open file.\n");
        exit(1);
    }
    
    print("File opened. FD=");
    print_u64(fd as u64);
    print("\nReading content...\n");
    
    // 内容を読み込む
    req.op = FsRequest::OP_READ;
    req.arg1 = fd as u64; // FD
    req.arg2 = 128;       // Length
    
    let req_slice = unsafe {
        core::slice::from_raw_parts(&req as *const _ as *const u8, core::mem::size_of::<FsRequest>())
    };
    ipc_send(fs_pid, req_slice);
    
    for _ in 0..10 {
        let (sender, len) = ipc_recv(&mut resp_buf);
        if sender == fs_pid && len >= core::mem::size_of::<FsResponse>() {
            let resp: FsResponse = unsafe { core::ptr::read(resp_buf.as_ptr() as *const _) };
            if resp.status > 0 {
                print("Read success:\n---\n");
                let data_len = resp.len as usize;
                if data_len > 0 && data_len <= 128 {
                    let s = core::str::from_utf8(&resp.data[..data_len]).unwrap_or("Invallid UTF-8");
                    print(s);
                }
                print("\n---\n");
            } else {
                print("Read failed.\n");
            }
            break;
        }
        yield_now();
    }
    
    print("TestApp finished.\n");
    exit(0);
}

/// 数値を文字列として出力（簡易実装）
fn print_u64(mut num: u64) {
    if num == 0 {
        print("0");
        return;
    }

    let mut buf = [0u8; 20];
    let mut i = 0;

    while num > 0 {
        buf[i] = (num % 10) as u8 + b'0';
        num /= 10;
        i += 1;
    }

    // 逆順で出力
    while i > 0 {
        i -= 1;
        let s = core::str::from_utf8(&buf[i..i+1]).unwrap();
        print(s);
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
