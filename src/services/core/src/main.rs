use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

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

const SERVICES: &[ServiceDef] = &[
    ServiceDef { name: "disk.service", path: "disk.service", order: 5 },
    ServiceDef { name: "fs.service", path: "fs.service", order: 10 },
];

#[cfg(feature = "run_tests")]
const TEST_APPS: &[TestApp] = &[
    TestApp { name: "tests", path: "/tests.elf" },
];

#[cfg(not(feature = "run_tests"))]
const TEST_APPS: &[TestApp] = &[];

/// サービスを起動する
fn start_service(service: &ServiceDef) -> Result<u32, String> {
    println!("[CORE] Starting service: {} (order={})", service.name, service.order);

    // std::process::Command を使用
    // SwiftCore側で fork/exec相当のシステムコールが実装されている前提
    match Command::new(service.path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn() 
    {
        Ok(child) => {
            let pid = child.id();
            println!("[CORE] Started {} with PID {}", service.name, pid);
            
            // std::thread::sleep を使用
            thread::sleep(Duration::from_millis(100));
            
            Ok(pid)
        }
        Err(e) => {
            let err_msg = format!("Failed to start {}: {}", service.name, e);
            Err(err_msg)
        }
    }
}

fn main() {
    println!("[CORE] Service Manager Started (using std)");

    // サービス起動
    for service in SERVICES {
        match start_service(service) {
            Ok(pid) => println!("[CORE] ✓ {} is running (PID={})", service.name, pid),
            Err(e) => println!("[CORE] ✗ {}", e),
        }
    }

    // テスト起動
    if !TEST_APPS.is_empty() {
        println!("[CORE] Starting test applications...");
        for test in TEST_APPS {
            println!("[CORE] Running test: {}", test.name);
            if let Err(e) = Command::new(test.path).spawn() {
                println!("[CORE] Failed to start test {}: {}", test.name, e);
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    println!("[CORE] Entering monitoring loop...");

    // 監視ループ
    loop {
        thread::sleep(Duration::from_secs(1));
        // TODO: ゾンビプロセス回収や再起動を行う
    }
}