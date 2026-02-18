//! ユーザー側システムコールスタブ
//!
//! 種類ごとのモジュールに分割し、ここから再エクスポートする。

pub mod ipc;
pub mod task;
pub mod time;
pub mod io;

// CランタイムとNewlibサポート
// mod crt; // crtは個別にコンパイルする
pub mod newlib;

mod sys;

pub use sys::SyscallNumber;
pub use ipc::{ipc_recv, ipc_send};
pub use task::{yield_now, exit, current_thread_id, thread_id_by_name};
pub use time::get_ticks;
pub use io::{exit, print, read, write, write_stderr, write_stdout, STDERR, STDIN, STDOUT};
