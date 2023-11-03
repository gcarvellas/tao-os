#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub fn kernel_main() -> () {
    loop {

    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
