use crate::interrupt::spinlock::SpinLock;

use super::ids::{PrivilegeLevel, ProcessId, ProcessState};

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
