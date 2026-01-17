//! ユーザー側システムコールスタブ
//!
//! 種類ごとのモジュールに分割し、ここから再エクスポートする。

pub mod ipc;
pub mod task;
pub mod time;
pub mod console;
pub mod fs;

mod sys;

pub use sys::SyscallNumber;
pub use ipc::{ipc_recv, ipc_send};
pub use task::{yield_now, exit};
pub use time::get_ticks;
pub use console::write as console_write;
pub use fs::read as initfs_read;
