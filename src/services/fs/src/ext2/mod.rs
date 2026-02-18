//! EXT2 ファイルシステム実装
//!
//! Linux標準のext2ファイルシステムをサポート

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::common::vfs::{
    DirEntry, FileAttr, FileSystem, FileType, VfsError, VfsResult,
};

/// ブロックデバイストレイト
///
/// 実際のストレージデバイスへのアクセスを抽象化
pub trait BlockDevice: Send + Sync {
    /// ブロックサイズ（通常512バイト）
    fn block_size(&self) -> usize;
    
    /// ブロックを読み取る
    fn read_block(&self, block_num: u64, buf: &mut [u8]) -> Result<(), ()>;
    
    /// ブロックに書き込む
    fn write_block(&mut self, block_num: u64, buf: &[u8]) -> Result<(), ()>;
}

/// EXT2スーパーブロック
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct Ext2Superblock {
    s_inodes_count: u32,        // inode総数
    s_blocks_count: u32,        // ブロック総数
    s_r_blocks_count: u32,      // 予約ブロック数
    s_free_blocks_count: u32,   // 空きブロック数
    s_free_inodes_count: u32,   // 空きinode数
    s_first_data_block: u32,    // 最初のデータブロック
    s_log_block_size: u32,      // ブロックサイズ (1024 << s_log_block_size)
    s_log_frag_size: u32,       // フラグメントサイズ
    s_blocks_per_group: u32,    // グループあたりブロック数
    s_frags_per_group: u32,     // グループあたりフラグメント数
    s_inodes_per_group: u32,    // グループあたりinode数
    s_mtime: u32,               // マウント時刻
    s_wtime: u32,               // 書き込み時刻
    s_mnt_count: u16,           // マウント回数
    s_max_mnt_count: u16,       // 最大マウント回数
    s_magic: u16,               // マジックナンバー (0xEF53)
    s_state: u16,               // ファイルシステム状態
    s_errors: u16,              // エラー時の動作
    s_minor_rev_level: u16,     // マイナーリビジョン
    s_lastcheck: u32,           // 最終チェック時刻
    s_checkinterval: u32,       // チェック間隔
    s_creator_os: u32,          // 作成OS
    s_rev_level: u32,           // リビジョンレベル
    s_def_resuid: u16,          // 予約ブロックのデフォルトUID
    s_def_resgid: u16,          // 予約ブロックのデフォルトGID
    // EXT2_DYNAMIC_REV (rev_level == 1) の追加フィールド
    s_first_ino: u32,           // 最初の使用可能inode
    s_inode_size: u16,          // inodeサイズ
    // ... その他のフィールドは省略
}

const EXT2_MAGIC: u16 = 0xEF53;
const EXT2_SUPERBLOCK_OFFSET: u64 = 1024;

/// EXT2 inode
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct Ext2Inode {
    i_mode: u16,                // ファイルモード
    i_uid: u16,                 // 所有者UID
    i_size: u32,                // サイズ（下位32ビット）
    i_atime: u32,               // アクセス時刻
    i_ctime: u32,               // 作成時刻
    i_mtime: u32,               // 変更時刻
    i_dtime: u32,               // 削除時刻
    i_gid: u16,                 // グループID
    i_links_count: u16,         // ハードリンク数
    i_blocks: u32,              // ブロック数
    i_flags: u32,               // フラグ
    i_osd1: u32,                // OS依存1
    i_block: [u32; 15],         // ブロックポインタ
    i_generation: u32,          // ファイルバージョン
    i_file_acl: u32,            // ファイルACL
    i_dir_acl: u32,             // ディレクトリACL
    i_faddr: u32,               // フラグメントアドレス
    i_osd2: [u8; 12],           // OS依存2
}

// inode モードフラグ
const EXT2_S_IFREG: u16 = 0x8000;   // 通常ファイル
const EXT2_S_IFDIR: u16 = 0x4000;   // ディレクトリ
const EXT2_S_IFLNK: u16 = 0xA000;   // シンボリックリンク

/// EXT2ディレクトリエントリ
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct Ext2DirEntry {
    inode: u32,         // inode番号
    rec_len: u16,       // このエントリのサイズ
    name_len: u8,       // 名前の長さ
    file_type: u8,      // ファイルタイプ
    // name: [u8]       // 可変長の名前（name_lenバイト）
}

/// EXT2ファイルシステム
pub struct Ext2Fs {
    device: Box<dyn BlockDevice>,
    superblock: Ext2Superblock,
    block_size: usize,
    inodes_per_group: u32,
    blocks_per_group: u32,
}

impl Ext2Fs {
    /// 新しいEXT2ファイルシステムを作成
    pub fn new(mut device: Box<dyn BlockDevice>) -> VfsResult<Self> {
        // スーパーブロックを読み取る
        let mut sb_buf = alloc::vec![0u8; 1024];
        device.read_block(EXT2_SUPERBLOCK_OFFSET / device.block_size() as u64, &mut sb_buf)
            .map_err(|_| VfsError::IoError)?;

        let superblock: Ext2Superblock = unsafe {
            core::ptr::read(sb_buf.as_ptr() as *const Ext2Superblock)
        };

        // マジックナンバーをチェック
        if superblock.s_magic != EXT2_MAGIC {
            return Err(VfsError::InvalidArgument);
        }

        let block_size = 1024 << superblock.s_log_block_size;

        Ok(Self {
            device,
            superblock,
            block_size,
            inodes_per_group: superblock.s_inodes_per_group,
            blocks_per_group: superblock.s_blocks_per_group,
        })
    }

    /// ブロックを読み取る
    fn read_fs_block(&self, block_num: u32, buf: &mut [u8]) -> VfsResult<()> {
        if buf.len() < self.block_size {
            return Err(VfsError::InvalidArgument);
        }

        // ファイルシステムブロックをデバイスブロックに変換
        let blocks_per_fs_block = self.block_size / self.device.block_size();
        let start_block = block_num as u64 * blocks_per_fs_block as u64;

        for i in 0..blocks_per_fs_block {
            let offset = i * self.device.block_size();
            self.device
                .read_block(start_block + i as u64, &mut buf[offset..offset + self.device.block_size()])
                .map_err(|_| VfsError::IoError)?;
        }

        Ok(())
    }

    /// inodeを読み取る
    fn read_inode(&self, inode_num: u64) -> VfsResult<Ext2Inode> {
        if inode_num == 0 {
            return Err(VfsError::NotFound);
        }

        // inodeが所属するブロックグループを計算
        let inode_idx = inode_num - 1;
        let group = inode_idx / self.inodes_per_group as u64;
        let local_idx = inode_idx % self.inodes_per_group as u64;

        // ブロックグループディスクリプタを読み取る（簡略化のため省略）
        // 実際にはブロックグループディスクリプタテーブルから
        // inode テーブルの開始ブロックを取得する必要がある

        // TODO: 実装を完成させる
        Err(VfsError::NotSupported)
    }
}

impl FileSystem for Ext2Fs {
    fn name(&self) -> &str {
        "ext2"
    }

    fn root_inode(&self) -> u64 {
        2 // ext2のルートinodeは常に2
    }

    fn stat(&self, inode: u64) -> VfsResult<FileAttr> {
        let ext2_inode = self.read_inode(inode)?;
        
        let file_type = match ext2_inode.i_mode & 0xF000 {
            EXT2_S_IFREG => FileType::RegularFile,
            EXT2_S_IFDIR => FileType::Directory,
            EXT2_S_IFLNK => FileType::SymbolicLink,
            _ => FileType::RegularFile,
        };

        Ok(FileAttr {
            file_type,
            size: ext2_inode.i_size as u64,
            blocks: ext2_inode.i_blocks as u64,
            atime: ext2_inode.i_atime as u64,
            mtime: ext2_inode.i_mtime as u64,
            ctime: ext2_inode.i_ctime as u64,
            mode: ext2_inode.i_mode,
            uid: ext2_inode.i_uid as u32,
            gid: ext2_inode.i_gid as u32,
            nlink: ext2_inode.i_links_count as u32,
        })
    }

    fn lookup(&self, _parent_inode: u64, _name: &str) -> VfsResult<u64> {
        // TODO: ディレクトリエントリを検索
        Err(VfsError::NotSupported)
    }

    fn read(&self, _inode: u64, _offset: u64, _buf: &mut [u8]) -> VfsResult<usize> {
        // TODO: ファイル読み取りを実装
        Err(VfsError::NotSupported)
    }

    fn write(&mut self, _inode: u64, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        // TODO: ファイル書き込みを実装（読み取り専用の場合はエラー）
        Err(VfsError::ReadOnlyFs)
    }

    fn readdir(&self, _inode: u64) -> VfsResult<Vec<DirEntry>> {
        // TODO: ディレクトリ読み取りを実装
        Err(VfsError::NotSupported)
    }

    fn create(&mut self, _parent_inode: u64, _name: &str, _mode: u16) -> VfsResult<u64> {
        Err(VfsError::ReadOnlyFs)
    }

    fn mkdir(&mut self, _parent_inode: u64, _name: &str, _mode: u16) -> VfsResult<u64> {
        Err(VfsError::ReadOnlyFs)
    }

    fn unlink(&mut self, _parent_inode: u64, _name: &str) -> VfsResult<()> {
        Err(VfsError::ReadOnlyFs)
    }

    fn rmdir(&mut self, _parent_inode: u64, _name: &str) -> VfsResult<()> {
        Err(VfsError::ReadOnlyFs)
    }

    fn truncate(&mut self, _inode: u64, _size: u64) -> VfsResult<()> {
        Err(VfsError::ReadOnlyFs)
    }

    fn sync(&mut self) -> VfsResult<()> {
        Ok(())
    }
}
