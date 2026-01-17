/// タスク関連システムコール
pub fn yield_now() -> u64 {
	crate::task::yield_now();
	0
}

/// 現在のスレッドを終了
pub fn exit(_code: u64) -> u64 {
	if let Some(id) = crate::task::current_thread_id() {
		crate::task::terminate_thread(id);
		0
	} else {
		crate::syscall::EINVAL
	}
}
