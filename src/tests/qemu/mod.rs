use crate::io::isr::outb;
use crate::println;
use core::fmt::Arguments;
use spin::Lazy;
use spin::Mutex;

pub mod serial_writer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    unsafe {
        outb(0xf4, exit_code as u8);
    }
    unreachable!()
}
