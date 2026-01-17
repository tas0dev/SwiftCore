use super::ids::ThreadId;
use super::thread::THREAD_QUEUE;

/// CPUコンテキスト（レジスタ保存用）
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Context {
    /// スタックポインタ
    pub rsp: u64,
    /// ベースポインタ
    pub rbp: u64,
    /// Callee-saved レジスタ
    pub rbx: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    /// 命令ポインタ（戻り先アドレス）
    pub rip: u64,
    /// RFLAGSレジスタ
    pub rflags: u64,
}

impl Context {
    /// 新しいコンテキストを作成
    pub const fn new() -> Self {
        Self {
            rsp: 0,
            rbp: 0,
            rbx: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: 0,
            rflags: 0,
        }
    }
}

/// コンテキストスイッチ
///
/// 現在のスレッドから次のスレッドへコンテキストを切り替える
///
/// Context構造体のレイアウト:
/// offset 0x00: rsp
/// offset 0x08: rbp
/// offset 0x10: rbx
/// offset 0x18: r12
/// offset 0x20: r13
/// offset 0x28: r14
/// offset 0x30: r15
/// offset 0x38: rip
/// offset 0x40: rflags
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn switch_context(old_context: *mut Context, new_context: *const Context) {
    core::arch::naked_asm!(
        // コンテキストスイッチ中の割り込みを禁止
        "cli",
        // 現在のコンテキストを保存
        // 呼び出し元に戻った後の rsp を保存（ret 相当）
        "lea rax, [rsp + 0x08]",
        // UEFI (x86_64-unknown-uefi) は Microsoft x64 ABI: rcx, rdx が引数
        "mov [rcx + 0x00], rax", // rsp
        "mov [rcx + 0x08], rbp", // rbp
        "mov [rcx + 0x10], rbx", // rbx
        "mov [rcx + 0x18], r12", // r12
        "mov [rcx + 0x20], r13", // r13
        "mov [rcx + 0x28], r14", // r14
        "mov [rcx + 0x30], r15", // r15
        // 戻り先アドレス（call命令でスタックにpushされている）を保存
        "mov rax, [rsp]",
        "mov [rcx + 0x38], rax", // rip
        // RFLAGSを保存
        "pushfq",
        "pop rax",
        "mov [rcx + 0x40], rax", // rflags
        // 新しいコンテキストを復元
        "mov rax, [rdx + 0x38]", // 新しいrip
        "mov r11, [rdx + 0x40]", // 新しいrflags
        "mov rbx, [rdx + 0x10]", // rbx
        "mov r12, [rdx + 0x18]", // r12
        "mov r13, [rdx + 0x20]", // r13
        "mov r14, [rdx + 0x28]", // r14
        "mov r15, [rdx + 0x30]", // r15
        "mov rbp, [rdx + 0x08]", // rbp
        "mov rsp, [rdx + 0x00]", // rsp
        // RFLAGSを復元
        "push r11",
        "popfq",
        // 新しいripへジャンプ
        "jmp rax"
    );
}

/// 現在のスレッドから指定されたスレッドIDにコンテキストスイッチ
pub unsafe fn switch_to_thread(current_id: Option<ThreadId>, next_id: ThreadId) {
    crate::debug!(
        "switch_to_thread: current_id={:?}, next_id={:?}",
        current_id,
        next_id
    );

    let mut queue = THREAD_QUEUE.lock();

    // 現在のスレッドのコンテキストへのポインタを取得
    let old_context_ptr = if let Some(id) = current_id {
        if let Some(thread) = queue.get_mut(id) {
            let ptr = thread.context_mut() as *mut Context;
            crate::debug!(
                "  Current context ptr: {:p}, rsp={:#x}, rip={:#x}",
                ptr,
                thread.context().rsp,
                thread.context().rip
            );
            ptr
        } else {
            return; // 現在のスレッドが見つからない
        }
    } else {
        // 現在のスレッドがない場合（初回スイッチ）
        // ダミーのコンテキストを使用
        crate::debug!("  No current thread (initial switch)");
        core::ptr::null_mut()
    };

    // 次のスレッドのコンテキストへのポインタを取得
    let new_context_ptr = if let Some(thread) = queue.get(next_id) {
        let ptr = thread.context() as *const Context;
        crate::debug!(
            "  Next context ptr: {:p}, rsp={:#x}, rip={:#x}",
            ptr,
            thread.context().rsp,
            thread.context().rip
        );
        ptr
    } else {
        return; // 次のスレッドが見つからない
    };

    // ロックを解放してからコンテキストスイッチ
    drop(queue);

    crate::debug!("About to perform context switch...");

    // コンテキストスイッチを実行
    if old_context_ptr.is_null() {
        // 初回スイッチの場合、現在のコンテキストを保存せずにジャンプ
        crate::debug!("Initial context switch (no save)");
        let ctx = &*new_context_ptr;
        core::arch::asm!(
            "cli",
            "mov rsp, {rsp}",
            "mov rbp, {rbp}",
            "mov rbx, {rbx}",
            "mov r12, {r12}",
            "mov r13, {r13}",
            "mov r14, {r14}",
            "mov r15, {r15}",
            "push {rflags}",
            "popfq",
            // エントリへジャンプ
            "jmp {rip}",
            rsp = in(reg) ctx.rsp,
            rbp = in(reg) ctx.rbp,
            rbx = in(reg) ctx.rbx,
            r12 = in(reg) ctx.r12,
            r13 = in(reg) ctx.r13,
            r14 = in(reg) ctx.r14,
            r15 = in(reg) ctx.r15,
            rflags = in(reg) ctx.rflags,
            rip = in(reg) ctx.rip,
            options(noreturn)
        );
    } else {
        crate::debug!("Normal context switch (save and restore)");
        crate::debug!(
            "  Calling switch_context({:p}, {:p})",
            old_context_ptr,
            new_context_ptr
        );
        switch_context(old_context_ptr, new_context_ptr);
        crate::debug!("  Returned from switch_context");
    }
}
