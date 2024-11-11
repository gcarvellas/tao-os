#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::io::isr::outb;

use alloc::fmt::Write;
use core::fmt::Arguments;
use spin::Lazy;
use spin::Mutex;

/*
 * Testing using serial printing
 */
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::tests::qemu::serial_writer::print(format_args!($($arg)*)));
}

pub fn print(args: Arguments) {
    QEMU_WRITER
        .lock()
        .write_fmt(args)
        .expect("Failed to write to serial");
}

pub static QEMU_WRITER: Lazy<Mutex<QemuWriter>> = Lazy::new(|| Mutex::new(QemuWriter {}));

struct QemuWriter {}

impl Write for QemuWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }
        Ok(())
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        unsafe {
            outb(0x3f8, c as u8);
        }
        Ok(())
    }
}
