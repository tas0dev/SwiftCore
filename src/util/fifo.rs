//! FIFOバッファ実装
//!
//! 割込みハンドラとカーネルの間でデータをやり取りするための
//! リングバッファ

use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;

/// FIFOバッファ
pub struct Fifo<T: Copy, const N: usize> {
    buffer: Mutex<FifoInner<T, N>>,
}

struct FifoInner<T: Copy, const N: usize> {
    data: [Option<T>; N],
    write_pos: usize,
    read_pos: usize,
    count: usize,
}

impl<T: Copy, const N: usize> Fifo<T, N> {
    /// 新しいFIFOバッファを作成
    pub const fn new() -> Self {
        Self {
            buffer: Mutex::new(FifoInner {
                data: [None; N],
                write_pos: 0,
                read_pos: 0,
                count: 0,
            }),
        }
    }

    /// データを追加（キューの末尾に追加）
    pub fn push(&self, value: T) -> Result<(), T> {
        let mut inner = self.buffer.lock();
        if inner.count >= N {
            return Err(value); // バッファ満杯
        }

        let write_pos = inner.write_pos;
        inner.data[write_pos] = Some(value);
        inner.write_pos = (inner.write_pos + 1) % N;
        inner.count += 1;
        Ok(())
    }

    /// データを取り出し（キューの先頭から取得）
    pub fn pop(&self) -> Option<T> {
        let mut inner = self.buffer.lock();
        if inner.count == 0 {
            return None; // バッファ空
        }

        let read_pos = inner.read_pos;
        let value = inner.data[read_pos].take();
        inner.read_pos = (inner.read_pos + 1) % N;
        inner.count -= 1;
        value
    }

    /// バッファが空かどうか
    pub fn is_empty(&self) -> bool {
        self.buffer.lock().count == 0
    }

    /// バッファが満杯かどうか
    pub fn is_full(&self) -> bool {
        self.buffer.lock().count >= N
    }

    /// 現在のデータ数
    pub fn len(&self) -> usize {
        self.buffer.lock().count
    }
}
