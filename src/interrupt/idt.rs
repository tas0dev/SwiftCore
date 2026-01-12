//! IDT管理

use crate::mem::gdt;
use crate::sprintln;
use spin::Once;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

static IDT: Once<InterruptDescriptorTable> = Once::new();

/// IDTを初期化
pub fn init() {
    sprintln!("Initializing IDT...");

    let idt = IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();

        // 例外ハンドラ
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        // ダブルフォルトハンドラ（ISTを使用）
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        // タイマー割込み (IRQ 0 -> INT 0x20)
        idt[0x20].set_handler_fn(timer_interrupt_handler);

        // キーボード割込み (IRQ 1 -> INT 0x21)
        idt[0x21].set_handler_fn(keyboard_interrupt_handler);

        idt
    });

    idt.load();
    sprintln!("IDT loaded");
}

/// ブレークポイント例外ハンドラ
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    sprintln!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

/// ページフォルト例外ハンドラ
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    sprintln!("EXCEPTION: PAGE FAULT");
    sprintln!("Accessed Address: {:?}", Cr2::read());
    sprintln!("Error Code: {:?}", error_code);
    sprintln!("{:#?}", stack_frame);

    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}

/// ダブルフォルト例外ハンドラ
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT (code: {})\n{:#?}",
        error_code, stack_frame
    );
}

/// タイマー割込みハンドラ (IRQ 0)
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // タイマー割込み処理
    crate::interrupt::timer::handle_timer_interrupt();

    // PICにEOI送信
    unsafe {
        crate::interrupt::pic::notify_end_of_interrupt(0x20);
    }
}

/// キーボード割込みハンドラ (IRQ 1)
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    // キーボードスキャンコードをFIFOに追加
    crate::interrupt::keyboard::handle_keyboard_interrupt(scancode);

    // PICにEOI送信
    unsafe {
        crate::interrupt::pic::notify_end_of_interrupt(0x21);
    }
}
