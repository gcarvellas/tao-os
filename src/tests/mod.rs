mod fat16_test;
mod malloc_test;
mod paging_test;
pub mod qemu;
use crate::kernel_init;
use crate::println;
use crate::tests::fat16_test::fat16_test;
use crate::tests::malloc_test::malloc_test;
use crate::tests::paging_test::paging_test;
use qemu::{exit_qemu, QemuExitCode};

pub fn test_main() -> ! {
    kernel_init();

    println!("Begin tests...");
    malloc_test();
    fat16_test().unwrap();
    paging_test().unwrap();
    exit_qemu(QemuExitCode::Success);
}

pub fn on_panic() -> ! {
    exit_qemu(QemuExitCode::Failed);
}
