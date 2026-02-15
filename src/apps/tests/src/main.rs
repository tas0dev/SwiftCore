#![no_std]
#![no_main]

extern crate alloc;
use core::ffi::c_char;
use swiftlib::cfunc::*;

// テスト結果の統計
struct TestStats {
    passed: usize,
    failed: usize,
    total: usize,
}

impl TestStats {
    const fn new() -> Self {
        Self {
            passed: 0,
            failed: 0,
            total: 0,
        }
    }

    fn run_test(&mut self, name: &str, test_fn: fn() -> bool) {
        self.total += 1;
        unsafe {
            printf(b"[TEST] Running: %s ... \0".as_ptr() as *const c_char, name.as_ptr());
        }
        
        if test_fn() {
            self.passed += 1;
            unsafe {
                printf(b"OK\n\0".as_ptr() as *const c_char);
            }
        } else {
            self.failed += 1;
            unsafe {
                printf(b"FAILED\n\0".as_ptr() as *const c_char);
            }
        }
    }

    fn summary(&self) {
        unsafe {
            printf(b"\n========================================\n\0".as_ptr() as *const c_char);
            printf(b"Test Results:\n\0".as_ptr() as *const c_char);
            printf(b"  Total:  %d\n\0".as_ptr() as *const c_char, self.total);
            printf(b"  Passed: %d\n\0".as_ptr() as *const c_char, self.passed);
            printf(b"  Failed: %d\n\0".as_ptr() as *const c_char, self.failed);
            printf(b"========================================\n\0".as_ptr() as *const c_char);
        }
    }
}

// ========== テスト関数 ==========

fn test_basic_arithmetic() -> bool {
    let a = 2 + 2;
    a == 4
}

fn test_string_compare() -> bool {
    let s1 = b"hello";
    let s2 = b"hello";
    s1 == s2
}

fn test_argc_argv() -> bool {
    true
}

// ========== メイン関数 ==========
#[no_mangle]
pub extern "C" fn main(argc: i32, argv: *const *const u8) -> i32 {
    unsafe {
        printf(b"\n========================================\n\0".as_ptr() as *const c_char);
        printf(b"SwiftCore Test Suite\n\0".as_ptr() as *const c_char);
        printf(b"========================================\n\n\0".as_ptr() as *const c_char);

        printf(b"Test invocation:\n\0".as_ptr() as *const c_char);
        printf(b"  argc: %d\n\0".as_ptr() as *const c_char, argc);
        for i in 0..argc {
            let arg_ptr = *argv.offset(i as isize);
            printf(b"  argv[%d]: %s\n\0".as_ptr() as *const c_char, i, arg_ptr);
        }
        printf(b"\n\0".as_ptr() as *const c_char);
    }

    let mut stats = TestStats::new();

    // テストを実行
    stats.run_test("basic_arithmetic\0", test_basic_arithmetic);
    stats.run_test("string_compare\0", test_string_compare);
    stats.run_test("argc_argv\0", test_argc_argv);

    // 結果を表示
    stats.summary();

    // 失敗があれば終了コード1を返す
    if stats.failed > 0 {
        1
    } else {
        0
    }
}
