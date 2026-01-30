/// 時刻関連システムコール
pub fn get_ticks() -> u64 {
    crate::interrupt::timer::get_ticks()
}
