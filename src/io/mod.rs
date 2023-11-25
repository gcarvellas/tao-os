pub mod vga;
pub mod isr;

use SCREEN;
use core::fmt::Arguments;

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
