//! PIC管理

use spin::Mutex;
use x86_64::instructions::port::Port;

/// PICのオフセット（IRQ番号をINT番号に変換）
pub const PIC1_OFFSET: u8 = 0x20; // IRQ 0-7 -> INT 0x20-0x27
pub const PIC2_OFFSET: u8 = 0x28; // IRQ 8-15 -> INT 0x28-0x2F

/// 8259 PICペア
pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC1_OFFSET, PIC2_OFFSET) });

/// マスターとスレーブの2つのPIC
pub struct ChainedPics {
    master: Pic,
    slave: Pic,
}

impl ChainedPics {
    /// 新しいPICペアを作成
    pub const unsafe fn new(master_offset: u8, slave_offset: u8) -> Self {
        Self {
            master: Pic::new(0x20, master_offset),
            slave: Pic::new(0xA0, slave_offset),
        }
    }

    /// PICを初期化
    pub unsafe fn initialize(&mut self) {
        let mut wait_port: Port<u8> = Port::new(0x80);
        let mut wait = || wait_port.write(0);

        // 初期化コマンド送信
        self.master.command.write(0x11);
        wait();
        self.slave.command.write(0x11);
        wait();

        // オフセット設定
        self.master.data.write(self.master.offset);
        wait();
        self.slave.data.write(self.slave.offset);
        wait();

        // カスケード設定
        self.master.data.write(4); // スレーブはIRQ2に接続
        wait();
        self.slave.data.write(2); // スレーブID = 2
        wait();

        // 8086モード
        self.master.data.write(0x01);
        wait();
        self.slave.data.write(0x01);
        wait();

        // すべての割込みをマスク（無効化）
        self.master.data.write(0xFF);
        wait();
        self.slave.data.write(0xFF);
        wait();
    }

    /// タイマーとキーボード割込みを有効化
    pub unsafe fn enable_timer_and_keyboard(&mut self) {
        // IRQ 1（キーボード）のみ有効化（タイマーは一旦無効のまま）
        // マスクビット: 0=有効, 1=無効
        // 0xFD = 11111101 (IRQ 1 有効、他は無効)
        self.master.data.write(0xFD);
    }

    /// 割込み終了を通知（EOI: End Of Interrupt）
    pub unsafe fn notify_end_of_interrupt(&mut self, interrupt_id: u8) {
        if interrupt_id >= self.slave.offset {
            // スレーブPICの割込みの場合、両方にEOIを送る
            self.slave.command.write(0x20);
        }
        self.master.command.write(0x20);
    }
}

/// 割込み終了を通知
pub unsafe fn notify_end_of_interrupt(interrupt_id: u8) {
    PICS.lock().notify_end_of_interrupt(interrupt_id);
}

/// 単一のPIC
struct Pic {
    command: Port<u8>,
    data: Port<u8>,
    offset: u8,
}

impl Pic {
    const unsafe fn new(port: u16, offset: u8) -> Self {
        Self {
            command: Port::new(port),
            data: Port::new(port + 1),
            offset,
        }
    }
}
