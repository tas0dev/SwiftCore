use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// ヒープサイズ (1MB) - プロセスごとのヒープサイズ
// 将来的にはbrk/sbrkシステムコールで動的に拡張するようにする
const HEAP_SIZE: usize = 1024 * 1024;

// bssセクションに確保される静的配列をヒープとして利用
static mut HEAP_MEM: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

/// ヒープアロケータを初期化する
/// プロセス開始時に一度だけ呼ぶ必要がある
pub fn init() {
    unsafe {
        ALLOCATOR.lock().init(HEAP_MEM.as_mut_ptr(), HEAP_SIZE);
    }
}

