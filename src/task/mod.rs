//! タスク管理モジュール
//!
//! マルチタスク機能を提供（プロセスとスレッドの管理）

use crate::interrupt::spinlock::SpinLock;
use core::sync::atomic::{AtomicU64, Ordering};

/// スレッド終了時に呼ばれるハンドラ
/// この関数から戻ることはない
extern "C" fn thread_exit_handler() -> ! {
    // スレッドが終了した場合の処理
    // 通常はここに到達することはない
    loop {
        x86_64::instructions::hlt();
    }
}

/// プロセスID生成用カウンタ
static NEXT_PROCESS_ID: AtomicU64 = AtomicU64::new(1);

/// スレッドID生成用カウンタ
static NEXT_THREAD_ID: AtomicU64 = AtomicU64::new(1);

/// プロセスID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessId(u64);

impl ProcessId {
    /// 新しいプロセスIDを生成
    pub fn new() -> Self {
        Self(NEXT_PROCESS_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// プロセスIDの値を取得
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// スレッドID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThreadId(u64);

impl ThreadId {
    /// 新しいスレッドIDを生成
    pub fn new() -> Self {
        Self(NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// スレッドIDの値を取得
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// スレッドの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
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

/// プロセスの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// 実行中（少なくとも1つのスレッドがRunning/Ready）
    Running,
    /// スリープ中（すべてのスレッドがSleeping）
    Sleeping,
    /// ゾンビ（終了したが親に回収されていない）
    Zombie,
    /// 終了済み
    Terminated,
}

/// タスクが保有する権限レベル。ServiceとUserは区別のためであり、両方ともRing3で動作する。
///
/// - Core: カーネルモード（Ring0）で動作するタスク。システムの中核機能を担当。
/// - Service: ユーザーモード（Ring3）で動作するが、システムサービスやドライバを担当。
/// - User: ユーザーモード（Ring3）で動作。一般的なアプリケーションを担当。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegeLevel {
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

/// プロセス構造体
///
/// メモリ空間とリソースを管理する実行単位。
/// 1つ以上のスレッドを持つ。
pub struct Process {
    /// プロセスID
    id: ProcessId,
    /// プロセス名
    name: &'static str,
    /// プロセスの状態
    state: ProcessState,
    /// 権限レベル
    privilege: PrivilegeLevel,
    /// 親プロセスID（存在する場合）
    parent_id: Option<ProcessId>,
    /// ページテーブルのアドレス（メモリ空間）。Noneの場合はカーネル空間を共有。
    page_table: Option<u64>,
    /// 優先度（0が最高、値が大きいほど低い）
    priority: u8,
}

impl Process {
    /// 新しいプロセスを作成
    ///
    /// # Arguments
    /// * `name` - プロセス名
    /// * `privilege` - 権限レベル
    /// * `parent_id` - 親プロセスID
    /// * `priority` - プロセスの優先度
    pub fn new(
        name: &'static str,
        privilege: PrivilegeLevel,
        parent_id: Option<ProcessId>,
        priority: u8,
    ) -> Self {
        Self {
            id: ProcessId::new(),
            name,
            state: ProcessState::Running,
            privilege,
            parent_id,
            page_table: None, // TODO: ページテーブル実装後に設定
            priority,
        }
    }

    /// プロセスIDを取得
    pub fn id(&self) -> ProcessId {
        self.id
    }

    /// プロセス名を取得
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// プロセスの状態を取得
    pub fn state(&self) -> ProcessState {
        self.state
    }

    /// プロセスの状態を設定
    pub fn set_state(&mut self, state: ProcessState) {
        self.state = state;
    }

    /// 権限レベルを取得
    pub fn privilege(&self) -> PrivilegeLevel {
        self.privilege
    }

    /// 親プロセスIDを取得
    pub fn parent_id(&self) -> Option<ProcessId> {
        self.parent_id
    }

    /// 優先度を取得
    pub fn priority(&self) -> u8 {
        self.priority
    }

    /// ページテーブルアドレスを取得
    pub fn page_table(&self) -> Option<u64> {
        self.page_table
    }

    /// ページテーブルアドレスを設定
    pub fn set_page_table(&mut self, page_table: u64) {
        self.page_table = Some(page_table);
    }
}

impl core::fmt::Debug for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut debug_struct = f.debug_struct("Process");
        debug_struct
            .field("id", &self.id)
            .field("name", &self.name)
            .field("state", &self.state)
            .field("privilege", &self.privilege)
            .field("parent_id", &self.parent_id)
            .field("priority", &self.priority);

        if let Some(pt) = self.page_table {
            debug_struct.field("page_table", &format_args!("{:#x}", pt));
        } else {
            debug_struct.field("page_table", &None::<u64>);
        }

        debug_struct.finish()
    }
}

/// スレッド構造体
///
/// プロセス内で実行される軽量な実行単位。
/// 同じプロセス内のスレッドはメモリ空間を共有する。
pub struct Thread {
    /// スレッドID
    id: ThreadId,
    /// 所属するプロセスID
    process_id: ProcessId,
    /// スレッド名
    name: &'static str,
    /// 現在の状態
    state: ThreadState,
    /// CPUコンテキスト
    context: Context,
    /// カーネルスタックの開始アドレス
    kernel_stack: u64,
    /// カーネルスタックのサイズ
    kernel_stack_size: usize,
}

impl Thread {
    /// 新しいスレッドを作成
    ///
    /// # Arguments
    /// * `process_id` - 所属するプロセスID
    /// * `name` - スレッド名
    /// * `entry_point` - スレッドのエントリーポイント関数
    /// * `kernel_stack` - カーネルスタックのアドレス
    /// * `kernel_stack_size` - カーネルスタックのサイズ
    pub fn new(
        process_id: ProcessId,
        name: &'static str,
        entry_point: fn() -> !,
        kernel_stack: u64,
        kernel_stack_size: usize,
    ) -> Self {
        let mut context = Context::new();

        // スタックポインタをスタックの最後に設定（スタックは下に伸びる）
        let stack_top = kernel_stack + kernel_stack_size as u64;

        // スタック上にダミーリターンアドレスを配置
        let stack_ptr = stack_top - 16;

        unsafe {
            let stack_addr = stack_ptr as *mut u64;
            // thread_exit_handlerアドレスをスタック上に配置
            let entry_addr = stack_ptr as *mut u64;
            *entry_addr = entry_point as u64;
            *entry_addr.add(1) = thread_exit_handler as u64;
        }

        // rscanにはダミーリターンアドレスを含んだ位置を設定
        context.rsp = stack_ptr;
        context.rbp = stack_top;

        // エントリーポイントをripに設定
        context.rip = entry_point as u64;

        // RFLAGSの初期値（割り込み有効）
        context.rflags = 0x202; // IF (Interrupt Flag) = 1

        crate::debug!(
            "Creating thread '{}': stack={:#x}, size={:#x}, rsp={:#x}, rip={:#x}",
            name,
            kernel_stack,
            kernel_stack_size,
            context.rsp,
            context.rip
        );

        Self {
            id: ThreadId::new(),
            process_id,
            name,
            state: ThreadState::Ready,
            context,
            kernel_stack,
            kernel_stack_size,
        }
    }

    /// スレッドIDを取得
    pub fn id(&self) -> ThreadId {
        self.id
    }

    /// 所属するプロセスIDを取得
    pub fn process_id(&self) -> ProcessId {
        self.process_id
    }

    /// スレッド名を取得
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// スレッドの状態を取得
    pub fn state(&self) -> ThreadState {
        self.state
    }

    /// スレッドの状態を設定
    pub fn set_state(&mut self, state: ThreadState) {
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
}

impl core::fmt::Debug for Thread {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Thread")
            .field("id", &self.id)
            .field("process_id", &self.process_id)
            .field("name", &self.name)
            .field("state", &self.state)
            .field("kernel_stack", &format_args!("{:#x}", self.kernel_stack))
            .field("kernel_stack_size", &self.kernel_stack_size)
            .finish()
    }
}

/// プロセステーブル
///
/// システム内のすべてのプロセスを管理する
pub struct ProcessTable {
    /// プロセスの配列（最大容量）
    processes: [Option<Process>; Self::MAX_PROCESSES],
    /// 現在のプロセス数
    count: usize,
}

impl ProcessTable {
    /// プロセステーブルの最大容量
    pub const MAX_PROCESSES: usize = 256;

    /// 新しいプロセステーブルを作成
    pub const fn new() -> Self {
        const INIT: Option<Process> = None;
        Self {
            processes: [INIT; Self::MAX_PROCESSES],
            count: 0,
        }
    }

    /// プロセスを追加
    ///
    /// # Returns
    /// 成功時はプロセスIDを返す。テーブルが満杯の場合はNone
    pub fn add(&mut self, process: Process) -> Option<ProcessId> {
        if self.count >= Self::MAX_PROCESSES {
            return None;
        }

        let id = process.id();

        // 空きスロットを探す
        for slot in &mut self.processes {
            if slot.is_none() {
                *slot = Some(process);
                self.count += 1;
                return Some(id);
            }
        }

        None
    }

    /// プロセスIDでプロセスを取得
    pub fn get(&self, id: ProcessId) -> Option<&Process> {
        self.processes
            .iter()
            .find_map(|slot| slot.as_ref().filter(|p| p.id() == id))
    }

    /// プロセスIDでプロセスの可変参照を取得
    pub fn get_mut(&mut self, id: ProcessId) -> Option<&mut Process> {
        self.processes
            .iter_mut()
            .find_map(|slot| slot.as_mut().filter(|p| p.id() == id))
    }

    /// プロセスを削除
    ///
    /// # Returns
    /// 削除されたプロセスを返す。存在しない場合はNone
    pub fn remove(&mut self, id: ProcessId) -> Option<Process> {
        for slot in &mut self.processes {
            if let Some(ref process) = slot {
                if process.id() == id {
                    self.count -= 1;
                    return slot.take();
                }
            }
        }
        None
    }

    /// すべてのプロセスを反復処理
    pub fn iter(&self) -> impl Iterator<Item = &Process> {
        self.processes.iter().filter_map(|slot| slot.as_ref())
    }

    /// すべてのプロセスを可変反復処理
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Process> {
        self.processes.iter_mut().filter_map(|slot| slot.as_mut())
    }

    /// 現在のプロセス数を取得
    pub fn count(&self) -> usize {
        self.count
    }

    /// プロセステーブルが満杯かどうか
    pub fn is_full(&self) -> bool {
        self.count >= Self::MAX_PROCESSES
    }

    /// プロセステーブルが空かどうか
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// グローバルプロセステーブル
static PROCESS_TABLE: SpinLock<ProcessTable> = SpinLock::new(ProcessTable::new());

/// プロセステーブルにプロセスを追加
pub fn add_process(process: Process) -> Option<ProcessId> {
    PROCESS_TABLE.lock().add(process)
}

/// プロセスIDでプロセス情報を取得（読み取り専用操作）
pub fn with_process<F, R>(id: ProcessId, f: F) -> Option<R>
where
    F: FnOnce(&Process) -> R,
{
    let table = PROCESS_TABLE.lock();
    table.get(id).map(f)
}

/// プロセスIDでプロセス情報を可変操作
pub fn with_process_mut<F, R>(id: ProcessId, f: F) -> Option<R>
where
    F: FnOnce(&mut Process) -> R,
{
    let mut table = PROCESS_TABLE.lock();
    table.get_mut(id).map(f)
}

/// プロセスを削除
pub fn remove_process(id: ProcessId) -> Option<Process> {
    PROCESS_TABLE.lock().remove(id)
}

/// すべてのプロセスに対して操作を実行
pub fn for_each_process<F>(mut f: F)
where
    F: FnMut(&Process),
{
    let table = PROCESS_TABLE.lock();
    for process in table.iter() {
        f(process);
    }
}

/// 現在のプロセス数を取得
pub fn process_count() -> usize {
    PROCESS_TABLE.lock().count()
}

/// スレッドキュー
///
/// 実行可能なスレッドを管理するキュー
pub struct ThreadQueue {
    /// スレッドの配列（最大容量）
    threads: [Option<Thread>; Self::MAX_THREADS],
    /// 現在のスレッド数
    count: usize,
}

impl ThreadQueue {
    /// スレッドキューの最大容量
    pub const MAX_THREADS: usize = 1024;

    /// 新しいスレッドキューを作成
    pub const fn new() -> Self {
        const INIT: Option<Thread> = None;
        Self {
            threads: [INIT; Self::MAX_THREADS],
            count: 0,
        }
    }

    /// スレッドを追加
    ///
    /// # Returns
    /// 成功時はスレッドIDを返す。キューが満杯の場合はNone
    pub fn push(&mut self, thread: Thread) -> Option<ThreadId> {
        if self.count >= Self::MAX_THREADS {
            return None;
        }

        let id = thread.id();

        // 空きスロットを探す
        for slot in &mut self.threads {
            if slot.is_none() {
                *slot = Some(thread);
                self.count += 1;
                return Some(id);
            }
        }

        None
    }

    /// スレッドIDでスレッドを取得
    pub fn get(&self, id: ThreadId) -> Option<&Thread> {
        self.threads
            .iter()
            .find_map(|slot| slot.as_ref().filter(|t| t.id() == id))
    }

    /// スレッドIDでスレッドの可変参照を取得
    pub fn get_mut(&mut self, id: ThreadId) -> Option<&mut Thread> {
        self.threads
            .iter_mut()
            .find_map(|slot| slot.as_mut().filter(|t| t.id() == id))
    }

    /// スレッドを削除
    ///
    /// # Returns
    /// 削除されたスレッドを返す。存在しない場合はNone
    pub fn remove(&mut self, id: ThreadId) -> Option<Thread> {
        for slot in &mut self.threads {
            if let Some(ref thread) = slot {
                if thread.id() == id {
                    self.count -= 1;
                    return slot.take();
                }
            }
        }
        None
    }

    /// 次に実行すべきスレッドを取得（削除せずに参照を返す）
    ///
    /// Ready状態のスレッドを優先して返す
    pub fn peek_next(&self) -> Option<&Thread> {
        // Ready状態のスレッドを探す
        self.threads
            .iter()
            .filter_map(|slot| slot.as_ref())
            .find(|t| t.state() == ThreadState::Ready)
    }

    /// 次に実行すべきスレッドを取得（可変参照）
    pub fn peek_next_mut(&mut self) -> Option<&mut Thread> {
        // Ready状態のスレッドを探す
        self.threads
            .iter_mut()
            .filter_map(|slot| slot.as_mut())
            .find(|t| t.state() == ThreadState::Ready)
    }

    /// 指定されたスレッドの次のReady状態のスレッドを取得（ラウンドロビン用）
    ///
    /// current_idの次のスロットから検索を開始し、見つからなければ先頭から検索
    pub fn peek_next_after(&mut self, current_id: Option<ThreadId>) -> Option<&mut Thread> {
        if let Some(current) = current_id {
            // 現在のスレッドのインデックスを探す
            let mut current_index = None;
            for (i, slot) in self.threads.iter().enumerate() {
                if let Some(thread) = slot.as_ref() {
                    if thread.id() == current {
                        current_index = Some(i);
                        break;
                    }
                }
            }

            if let Some(start_idx) = current_index {
                // 現在のインデックスの次から検索
                for i in (start_idx + 1)..Self::MAX_THREADS {
                    if let Some(thread) = &self.threads[i] {
                        if thread.state() == ThreadState::Ready {
                            // インデックスを使って可変参照を返す
                            return self.threads[i].as_mut();
                        }
                    }
                }

                // 見つからなければ先頭から現在のインデックスまで検索
                for i in 0..=start_idx {
                    if let Some(thread) = &self.threads[i] {
                        if thread.state() == ThreadState::Ready {
                            // インデックスを使って可変参照を返す
                            return self.threads[i].as_mut();
                        }
                    }
                }
            }
        }

        // current_idがない場合は最初のReady状態のスレッドを返す
        self.peek_next_mut()
    }

    /// 指定された状態のスレッド数をカウント
    pub fn count_by_state(&self, state: ThreadState) -> usize {
        self.threads
            .iter()
            .filter_map(|slot| slot.as_ref())
            .filter(|t| t.state() == state)
            .count()
    }

    /// 指定されたプロセスに属するスレッドを反復処理
    pub fn iter_by_process(&self, process_id: ProcessId) -> impl Iterator<Item = &Thread> {
        self.threads
            .iter()
            .filter_map(|slot| slot.as_ref())
            .filter(move |t| t.process_id() == process_id)
    }

    /// すべてのスレッドを反復処理
    pub fn iter(&self) -> impl Iterator<Item = &Thread> {
        self.threads.iter().filter_map(|slot| slot.as_ref())
    }

    /// すべてのスレッドを可変反復処理
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Thread> {
        self.threads.iter_mut().filter_map(|slot| slot.as_mut())
    }

    /// 現在のスレッド数を取得
    pub fn count(&self) -> usize {
        self.count
    }

    /// スレッドキューが満杯かどうか
    pub fn is_full(&self) -> bool {
        self.count >= Self::MAX_THREADS
    }

    /// スレッドキューが空かどうか
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// グローバルスレッドキュー
static THREAD_QUEUE: SpinLock<ThreadQueue> = SpinLock::new(ThreadQueue::new());

/// 現在実行中のスレッドID
static CURRENT_THREAD: SpinLock<Option<ThreadId>> = SpinLock::new(None);

/// スレッドキューにスレッドを追加
pub fn add_thread(thread: Thread) -> Option<ThreadId> {
    THREAD_QUEUE.lock().push(thread)
}

/// スレッドIDでスレッド情報を取得（読み取り専用操作）
pub fn with_thread<F, R>(id: ThreadId, f: F) -> Option<R>
where
    F: FnOnce(&Thread) -> R,
{
    let queue = THREAD_QUEUE.lock();
    queue.get(id).map(f)
}

/// スレッドIDでスレッド情報を可変操作
pub fn with_thread_mut<F, R>(id: ThreadId, f: F) -> Option<R>
where
    F: FnOnce(&mut Thread) -> R,
{
    let mut queue = THREAD_QUEUE.lock();
    queue.get_mut(id).map(f)
}

/// スレッドを削除
pub fn remove_thread(id: ThreadId) -> Option<Thread> {
    THREAD_QUEUE.lock().remove(id)
}

/// 次に実行すべきスレッドIDを取得
pub fn peek_next_thread() -> Option<ThreadId> {
    THREAD_QUEUE.lock().peek_next().map(|t| t.id())
}

/// 指定された状態のスレッド数を取得
pub fn count_threads_by_state(state: ThreadState) -> usize {
    THREAD_QUEUE.lock().count_by_state(state)
}

/// すべてのスレッドに対して操作を実行
pub fn for_each_thread<F>(mut f: F)
where
    F: FnMut(&Thread),
{
    let queue = THREAD_QUEUE.lock();
    for thread in queue.iter() {
        f(thread);
    }
}

/// 現在のスレッド数を取得
pub fn thread_count() -> usize {
    THREAD_QUEUE.lock().count()
}

/// 現在実行中のスレッドIDを取得
pub fn current_thread_id() -> Option<ThreadId> {
    *CURRENT_THREAD.lock()
}

/// 現在実行中のスレッドIDを設定
pub fn set_current_thread(id: Option<ThreadId>) {
    *CURRENT_THREAD.lock() = id;
}

/// スケジューラ
///
/// スレッドのスケジューリングを管理
pub struct Scheduler {
    /// スケジューラが有効かどうか
    enabled: bool,
    /// タイムスライス（タイマー割り込み回数）
    time_slice: u64,
    /// 現在のタイムスライスカウンタ
    current_slice: u64,
}

impl Scheduler {
    /// デフォルトのタイムスライス（10ms × 10 = 100ms）
    pub const DEFAULT_TIME_SLICE: u64 = 10;

    /// 新しいスケジューラを作成
    pub const fn new() -> Self {
        Self {
            enabled: false,
            time_slice: Self::DEFAULT_TIME_SLICE,
            current_slice: 0,
        }
    }

    /// スケジューラを有効化
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// スケジューラを無効化
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// スケジューラが有効かどうか
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// タイムスライスを設定
    pub fn set_time_slice(&mut self, slice: u64) {
        self.time_slice = slice;
    }

    /// タイマー割り込み時に呼ばれる
    ///
    /// タイムスライスをカウントし、期限が来たらスケジューリングを実行
    pub fn tick(&mut self) -> bool {
        if !self.enabled {
            return false;
        }

        self.current_slice += 1;
        if self.current_slice >= self.time_slice {
            self.current_slice = 0;
            true // スケジューリングが必要
        } else {
            false
        }
    }

    /// タイムスライスをリセット
    pub fn reset_slice(&mut self) {
        self.current_slice = 0;
    }
}

/// グローバルスケジューラ
static SCHEDULER: SpinLock<Scheduler> = SpinLock::new(Scheduler::new());

/// スケジューラを初期化
pub fn init_scheduler() {
    let mut scheduler = SCHEDULER.lock();
    scheduler.enable();
}

/// スケジューラを有効化
pub fn enable_scheduler() {
    SCHEDULER.lock().enable();
}

/// スケジューラを無効化
pub fn disable_scheduler() {
    SCHEDULER.lock().disable();
}

/// スケジューラが有効かどうか
pub fn is_scheduler_enabled() -> bool {
    SCHEDULER.lock().is_enabled()
}

/// タイマー割り込み時に呼ばれる（タイマー割り込みハンドラから呼び出す）
///
/// # Returns
/// スケジューリングが必要な場合はtrue
pub fn scheduler_tick() -> bool {
    SCHEDULER.lock().tick()
}

/// 次に実行すべきスレッドを選択
///
/// ラウンドロビンスケジューリング：Ready状態のスレッドを順に選択
///
/// # Returns
/// 次に実行すべきスレッドID。実行可能なスレッドがない場合はNone
pub fn schedule() -> Option<ThreadId> {
    let mut queue = THREAD_QUEUE.lock();

    // 現在のスレッドを取得
    let current = *CURRENT_THREAD.lock();

    // 現在のスレッドがあれば、状態をReadyに戻す（Running -> Ready）
    if let Some(current_id) = current {
        if let Some(thread) = queue.get_mut(current_id) {
            if thread.state() == ThreadState::Running {
                thread.set_state(ThreadState::Ready);
            }
        }
    }

    // 現在のスレッドの次のReady状態のスレッドを探す
    if let Some(next_thread) = queue.peek_next_after(current) {
        let next_id = next_thread.id();
        next_thread.set_state(ThreadState::Running);

        // スケジューラのタイムスライスをリセット
        drop(queue);
        SCHEDULER.lock().reset_slice();

        Some(next_id)
    } else {
        None
    }
}

/// 現在のスレッドを明示的にCPUを手放す（yield）
///
/// スケジューラを呼び出して次のスレッドに切り替える
pub fn yield_now() {
    if !is_scheduler_enabled() {
        return;
    }

    crate::debug!("yield_now() called");

    // スケジューリングを実行
    if let Some(next_id) = schedule() {
        let current = current_thread_id();

        crate::debug!("yield_now: current={:?}, next={:?}", current, next_id);

        // 次のスレッドが現在のスレッドと異なる場合のみ切り替え
        if Some(next_id) != current {
            set_current_thread(Some(next_id));

            crate::debug!("Calling switch_to_thread...");

            // コンテキストスイッチを実行
            unsafe {
                switch_to_thread(current, next_id);
            }

            crate::debug!("Returned from switch_to_thread");
        }
    }
}

/// スレッドをブロック状態にする
///
/// 現在のスレッドをBlocked状態にして、次のスレッドにスケジューリング
pub fn block_current_thread() {
    if let Some(current_id) = current_thread_id() {
        with_thread_mut(current_id, |thread| {
            thread.set_state(ThreadState::Blocked);
        });

        // 次のスレッドにスケジューリング
        yield_now();
    }
}

/// スレッドをスリープ状態にする
///
/// 指定されたスレッドをSleeping状態にする
pub fn sleep_thread(id: ThreadId) {
    with_thread_mut(id, |thread| {
        thread.set_state(ThreadState::Sleeping);
    });
}

/// スレッドを起床させる
///
/// Sleeping/Blocked状態のスレッドをReady状態にする
pub fn wake_thread(id: ThreadId) {
    with_thread_mut(id, |thread| {
        let state = thread.state();
        if state == ThreadState::Sleeping || state == ThreadState::Blocked {
            thread.set_state(ThreadState::Ready);
        }
    });
}

/// スレッドを終了させる
///
/// 指定されたスレッドをTerminated状態にして削除
pub fn terminate_thread(id: ThreadId) {
    with_thread_mut(id, |thread| {
        thread.set_state(ThreadState::Terminated);
    });

    // 現在のスレッドの場合は次のスレッドにスケジューリング
    if Some(id) == current_thread_id() {
        set_current_thread(None);
        yield_now();
    }

    // スレッドをキューから削除
    remove_thread(id);
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
        "mov [rdi + 0x00], rsp",   // rsp
        "mov [rdi + 0x08], rbp",   // rbp
        "mov [rdi + 0x10], rbx",   // rbx
        "mov [rdi + 0x18], r12",   // r12
        "mov [rdi + 0x20], r13",   // r13
        "mov [rdi + 0x28], r14",   // r14
        "mov [rdi + 0x30], r15",   // r15

        // 戻り先アドレス（call命令でスタックにpushされている）を保存
        "mov rax, [rsp]",
        "mov [rdi + 0x38], rax",   // rip

        // RFLAGSを保存
        "pushfq",
        "pop rax",
        "mov [rdi + 0x40], rax",   // rflags

        // 新しいコンテキストを復元
        "mov rax, [rsi + 0x38]",   // 新しいrip
        "mov rcx, [rsi + 0x40]",   // 新しいrflags
        "mov rbx, [rsi + 0x10]",   // rbx
        "mov r12, [rsi + 0x18]",   // r12
        "mov r13, [rsi + 0x20]",   // r13
        "mov r14, [rsi + 0x28]",   // r14
        "mov r15, [rsi + 0x30]",   // r15
        "mov rbp, [rsi + 0x08]",   // rbp
        "mov rsp, [rsi + 0x00]",   // rsp

        // RFLAGSを復元
        "push rcx",
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

/// スケジューリングしてコンテキストスイッチを実行
///
/// タイマー割り込みハンドラから呼び出される
pub fn schedule_and_switch() {
    if !is_scheduler_enabled() {
        return;
    }

    let current = current_thread_id();

    // 次のスレッドを選択
    if let Some(next_id) = schedule() {
        // 次のスレッドが現在のスレッドと異なる場合のみ切り替え
        if Some(next_id) != current {
            set_current_thread(Some(next_id));

            // コンテキストスイッチを実行
            unsafe {
                switch_to_thread(current, next_id);
            }
        }
    }
}

/// 最初のスレッドを起動
///
/// スケジューラを開始して最初のスレッドにジャンプ
pub fn start_scheduling() -> ! {
    // 最初のスレッドを選択
    if let Some(first_id) = peek_next_thread() {
        set_current_thread(Some(first_id));

        // デバッグ情報を出力
        with_thread_mut(first_id, |thread| {
            crate::debug!(
                "Starting first thread: {} (id={:?})",
                thread.name(),
                thread.id()
            );
            crate::debug!(
                "  Context: rsp={:#x}, rip={:#x}, rflags={:#x}",
                thread.context().rsp,
                thread.context().rip,
                thread.context().rflags
            );
            thread.set_state(ThreadState::Running);
        });

        // 最初のスレッドにジャンプ（戻ってこない）
        unsafe {
            switch_to_thread(None, first_id);
        }

        unreachable!("switch_to_thread should never return");
    } else {
        panic!("No threads to schedule!");
    }
}
