#![no_std]
#![no_main]

extern crate alloc;

use core::fmt::{self, Write};
use swiftlib::io;
use swiftlib::task;
use swiftlib::process;

// 簡易的な標準出力ライター
struct Stdout;
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        io::write_stdout(s.as_bytes());
        Ok(())
    }
}

macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ({
        let _ = writeln!(&mut Stdout, $($arg)*);
    });
}

/// サービス定義
struct ServiceDef {
    name: &'static str,
    path: &'static str,
    order: u32,
}

/// テストアプリ定義
struct TestApp {
    name: &'static str,
    path: &'static str,
}

/// 起動するサービスのリスト（index.tomlから生成する想定）
/// 現在は静的に定義
const SERVICES: &[ServiceDef] = &[
    ServiceDef {
        name: "disk.service",
        path: "disk.service",
        order: 5,
    },
    ServiceDef {
        name: "fs.service",
        path: "fs.service",
        order: 10,
    },
    // 将来的には他のサービスも追加
    // ServiceDef { name: "net.service", path: "net.service", order: 20 },
];

/// テストアプリケーションのリスト
#[cfg(feature = "run_tests")]
const TEST_APPS: &[TestApp] = &[
    TestApp {
        name: "tests",
        path: "/tests.elf",
    },
];

#[cfg(not(feature = "run_tests"))]
const TEST_APPS: &[TestApp] = &[];

/// サービスを起動する
fn start_service(service: &ServiceDef) -> Result<u64, &'static str> {
    println!("[CORE] Starting service: {} (order={})", service.name, service.order);
    
    // execシステムコールを使用してサービスを起動
    match process::exec(service.path) {
        Ok(pid) => {
            println!("[CORE] Started {} with PID {}", service.name, pid);
            
            // サービスが初期化されるまで少し待つ
            task::sleep(100);
            
            Ok(pid)
        }
        Err(_) => {
            println!("[CORE] Failed to start {}", service.name);
            Err("exec failed")
        }
    }
}

#[no_mangle]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    println!("[CORE] Service Manager Started");
    
    // すべてのサービスをorder順に起動
    for service in SERVICES.iter() {
        match start_service(service) {
            Ok(pid) => {
                println!("[CORE] ✓ {} is running (PID={})", service.name, pid);
            }
            Err(e) => {
                println!("[CORE] ✗ Failed to start {}: {}", service.name, e);
            }
        }
    }
    
    println!("[CORE] All services started. Entering monitoring loop...");
    
    // テストアプリケーションを起動（feature有効時のみ）
    if !TEST_APPS.is_empty() {
        println!("[CORE] Starting test applications...");
        for test in TEST_APPS.iter() {
            println!("[CORE] Running test: {}", test.name);
            match process::exec(test.path) {
                Ok(pid) => {
                    println!("[CORE] Test {} started with PID {}", test.name, pid);
                }
                Err(_) => {
                    println!("[CORE] Failed to start test {}", test.name);
                }
            }
            task::sleep(100);
        }
        println!("[CORE] All tests started.");
    }
    
    // サービス監視ループ
    loop {
        task::sleep(1000); // 1秒ごとに監視
        // TODO: サービスの状態チェック
    }
}
