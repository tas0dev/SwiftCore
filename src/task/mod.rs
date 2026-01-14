//! タスク管理モジュール
//!
//! マルチタスク機能を提供

use crate::interrupt::spinlock::SpinLock;
use core::sync::atomic::{AtomicU64, Ordering};

/// タスクID生成用カウンタ
static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

/// タスクID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);

impl TaskId {
    /// 新しいタスクIDを生成
    pub fn new() -> Self {
        Self(NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// タスクIDの値を取得
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// タスクの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// 実行可能（スケジューラ待ち）
    Ready,
    /// 実行中
    Running,
    /// ブロック中（I/O待ちなど）
    Blocked,
    /// スリープ中
    Sleeping,
    /// 終了済み
    Terminated,
}

/// タスクが保有する権限レベル。ServiceとUserは区別のためであり、両方ともRing3で動作する。
///
/// - Core: カーネルモード（Ring0）で動作するタスク。システムの中核機能を担当。
/// - Service: ユーザーモード（Ring3）で動作するが、システムサービスやドライバを担当。
/// - User: ユーザーモード（Ring3）で動作。一般的なアプリケーションを担当。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskLevel {
    /// コアレベルタスク（Ring0）
    Core,
    /// サービスレベルタスク（Ring3）
    Service,
    /// ユーザーレベルタスク（Ring3）
    User,
}

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

/// タスク構造体
pub struct Task {
    /// タスクID
    id: TaskId,
    /// タスク名
    name: &'static str,
    /// 現在の状態
    state: TaskState,
    /// CPUコンテキスト
    context: Context,
    /// カーネルスタックの開始アドレス
    kernel_stack: u64,
    /// カーネルスタックのサイズ
    kernel_stack_size: usize,
    /// 優先度（0が最高、値が大きいほど低い）
    priority: u8,
    /// タスクのレベル（保有する権限）
    level: TaskLevel,
}

impl Task {
    /// 新しいタスクを作成
    ///
    /// # Arguments
    /// * `name` - タスク名
    /// * `entry_point` - タスクのエントリーポイント関数
    /// * `kernel_stack` - カーネルスタックのアドレス
    /// * `kernel_stack_size` - カーネルスタックのサイズ
    /// * `priority` - タスクの優先度
    /// * `level` - タスクのレベル（権限）
    pub fn new(
        name: &'static str,
        entry_point: fn() -> !,
        kernel_stack: u64,
        kernel_stack_size: usize,
        priority: u8,
        level: TaskLevel,
    ) -> Self {
        let mut context = Context::new();

        // スタックポインタをスタックの最後に設定（スタックは下に伸びる）
        context.rsp = kernel_stack + kernel_stack_size as u64;
        context.rbp = context.rsp;

        // エントリーポイントをripに設定
        context.rip = entry_point as u64;

        // RFLAGSの初期値（割り込み有効）
        context.rflags = 0x202; // IF (Interrupt Flag) = 1

        Self {
            id: TaskId::new(),
            name,
            state: TaskState::Ready,
            context,
            kernel_stack,
            kernel_stack_size,
            priority,
            level,
        }
    }

    /// タスクIDを取得
    pub fn id(&self) -> TaskId {
        self.id
    }

    /// タスク名を取得
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// タスクの状態を取得
    pub fn state(&self) -> TaskState {
        self.state
    }

    /// タスクの状態を設定
    pub fn set_state(&mut self, state: TaskState) {
        self.state = state;
    }

    /// コンテキストへの可変参照を取得
    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    /// コンテキストへの参照を取得
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// 優先度を取得
    pub fn priority(&self) -> u8 {
        self.priority
    }

    /// タスクのレベルを取得
    pub fn level(&self) -> TaskLevel {
        self.level
    }
}

impl core::fmt::Debug for Task {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("state", &self.state)
            .field("priority", &self.priority)
            .field("level", &self.level)
            .field("kernel_stack", &format_args!("{:#x}", self.kernel_stack))
            .field("kernel_stack_size", &self.kernel_stack_size)
            .finish()
    }
}

// TODO: タスクキューの実装
// TODO: スケジューラの実装
// TODO: コンテキストスイッチの実装
