//! PS/2キーボードドライバ

use spin::Mutex;
use x86_64::instructions::port::Port;

const DATA_PORT: u16 = 0x60;
const STATUS_PORT: u16 = 0x64;

const BUFFER_SIZE: usize = 128;

struct KeyBuffer {
    buf: [u8; BUFFER_SIZE],
    head: usize,
    tail: usize,
    len: usize,
}

impl KeyBuffer {
    const fn new() -> Self {
        Self {
            buf: [0; BUFFER_SIZE],
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    fn push(&mut self, byte: u8) {
        if self.len >= BUFFER_SIZE {
            return;
        }
        self.buf[self.tail] = byte;
        self.tail = (self.tail + 1) % BUFFER_SIZE;
        self.len += 1;
    }

    fn pop(&mut self) -> Option<u8> {
        if self.len == 0 {
            return None;
        }
        let byte = self.buf[self.head];
        self.head = (self.head + 1) % BUFFER_SIZE;
        self.len -= 1;
        Some(byte)
    }

    fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.len = 0;
    }
}

struct KeyboardState {
    shift: bool,
    caps: bool,
    extended: bool,
    buffer: KeyBuffer,
}

impl KeyboardState {
    const fn new() -> Self {
        Self {
            shift: false,
            caps: false,
            extended: false,
            buffer: KeyBuffer::new(),
        }
    }
}

static KEYBOARD: Mutex<KeyboardState> = Mutex::new(KeyboardState::new());

pub fn init() {
    KEYBOARD.lock().buffer.clear();
}

pub fn handle_interrupt() {
    let status = unsafe { Port::<u8>::new(STATUS_PORT).read() };
    if status & 0x01 == 0 {
        return;
    }

    let scancode = unsafe { Port::<u8>::new(DATA_PORT).read() };
    handle_scancode(scancode);
}

pub fn read_char() -> Option<u8> {
    KEYBOARD.lock().buffer.pop()
}

fn handle_scancode(scancode: u8) {
    let mut kbd = KEYBOARD.lock();

    if scancode == 0xE0 {
        kbd.extended = true;
        return;
    }

    let released = scancode & 0x80 != 0;
    let code = scancode & 0x7F;

    match code {
        0x2A | 0x36 => {
            kbd.shift = !released;
            kbd.extended = false;
            return;
        }
        0x3A if !released => {
            kbd.caps = !kbd.caps;
            kbd.extended = false;
            return;
        }
        _ => {}
    }

    if released {
        kbd.extended = false;
        return;
    }

    if kbd.extended {
        kbd.extended = false;
        return;
    }

    if let Some(ch) = scancode_to_ascii(code, kbd.shift, kbd.caps) {
        kbd.buffer.push(ch);
    }
}

fn scancode_to_ascii(code: u8, shift: bool, caps: bool) -> Option<u8> {
    let ch = match code {
        0x02 => b'1',
        0x03 => b'2',
        0x04 => b'3',
        0x05 => b'4',
        0x06 => b'5',
        0x07 => b'6',
        0x08 => b'7',
        0x09 => b'8',
        0x0A => b'9',
        0x0B => b'0',
        0x0C => b'-',
        0x0D => b'=',
        0x0E => 8,
        0x0F => b'\t',
        0x10 => b'q',
        0x11 => b'w',
        0x12 => b'e',
        0x13 => b'r',
        0x14 => b't',
        0x15 => b'y',
        0x16 => b'u',
        0x17 => b'i',
        0x18 => b'o',
        0x19 => b'p',
        0x1A => b'[',
        0x1B => b']',
        0x1C => b'\n',
        0x1E => b'a',
        0x1F => b's',
        0x20 => b'd',
        0x21 => b'f',
        0x22 => b'g',
        0x23 => b'h',
        0x24 => b'j',
        0x25 => b'k',
        0x26 => b'l',
        0x27 => b';',
        0x28 => b'\'',
        0x29 => b'`',
        0x2B => b'\\',
        0x2C => b'z',
        0x2D => b'x',
        0x2E => b'c',
        0x2F => b'v',
        0x30 => b'b',
        0x31 => b'n',
        0x32 => b'm',
        0x33 => b',',
        0x34 => b'.',
        0x35 => b'/',
        0x39 => b' ',
        _ => return None,
    };

    if ch.is_ascii_alphabetic() {
        let upper = caps ^ shift;
        if upper {
            return Some(ch.to_ascii_uppercase());
        }
        return Some(ch);
    }

    if !shift {
        return Some(ch);
    }

    let shifted = match ch {
        b'1' => b'!',
        b'2' => b'@',
        b'3' => b'#',
        b'4' => b'$',
        b'5' => b'%',
        b'6' => b'^',
        b'7' => b'&',
        b'8' => b'*',
        b'9' => b'(',
        b'0' => b')',
        b'-' => b'_',
        b'=' => b'+',
        b'[' => b'{',
        b']' => b'}',
        b'\\' => b'|',
        b';' => b':',
        b'\'' => b'"',
        b',' => b'<',
        b'.' => b'>',
        b'/' => b'?',
        b'`' => b'~',
        _ => ch,
    };

    Some(shifted)
}
