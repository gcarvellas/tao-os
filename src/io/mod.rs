pub mod isr;
pub mod vga;

use core::fmt::Arguments;

use self::vga::SCREEN;

/*
 * Testing using serial printing
 */
#[cfg(not(feature = "integration"))]
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
    SCREEN.lock().write_fmt(args).expect("Failed to print");
}
