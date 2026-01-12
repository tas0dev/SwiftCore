//! タイマー割込みハンドラ

use core::sync::atomic::{AtomicU64, Ordering};

/// システム起動からのタイマーティック数
static TICKS: AtomicU64 = AtomicU64::new(0);

/// タイマー割込みを処理
pub fn handle_timer_interrupt() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

/// 現在のティック数を取得
pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}
