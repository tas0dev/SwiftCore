/// タスク関連システムコール
pub fn yield_now() -> u64 {
	crate::task::yield_now();
	0
}
