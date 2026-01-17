//! ユーザー側システムコールスタブ
//!
//! 種類ごとのモジュールに分割し、ここから再エクスポートする。

pub mod ipc;
pub mod task;
pub mod time;

mod sys;

pub use sys::SyscallNumber;
pub use ipc::{ipc_recv, ipc_send};
pub use task::yield_now;
pub use time::get_ticks;
