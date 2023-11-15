#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr;

const VIDEO_ADDRESS: *mut u8 = 0xb8000 as *mut u8;

#[no_mangle]
pub fn kernel_main() -> () {
    // TODO for testing
    unsafe {
        ptr::write_volatile(VIDEO_ADDRESS.offset(0), 0x4f);
        ptr::write_volatile(VIDEO_ADDRESS.offset(1), 0x0e);
        ptr::write_volatile(VIDEO_ADDRESS.offset(2), 0x4b);
        ptr::write_volatile(VIDEO_ADDRESS.offset(3), 0x0e);
    }

    loop {

    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
