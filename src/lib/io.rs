use core::fmt::{self, Write};
use crate::sys::{syscall3, SyscallNumber};

pub struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            syscall3(
                SyscallNumber::Write as u64,
                1, // stdout fd
                s.as_ptr() as u64,
                s.len() as u64,
            );
        }
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    let mut out = Stdout;
    out.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

