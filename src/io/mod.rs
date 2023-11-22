pub mod vga;

use SCREEN;
use core::fmt::Arguments;

extern {
    fn _insb(port: u16) -> u8;
    fn _insw(port: u16) -> u16;
    fn _outb(port: u16, val: u8);
    fn _outw(port: u16, val: u16);
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

#[doc(hidden)]
pub fn _print(args: Arguments) {
    use core::fmt::Write;
    SCREEN.lock().write_fmt(args).unwrap();
}

pub fn insb(port: u16) -> u8 {
    return unsafe { _insb(port) };
}

pub fn insw(port: u16) -> u16 {
    return unsafe { _insw(port) };
}

pub fn outb(port: u16, val: u8) {
    unsafe { _outb(port, val) };
}

pub fn outw(port: u16, val: u16) {
    unsafe { _outw(port, val) };
}
