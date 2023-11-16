#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr;

const VIDEO_ADDRESS: *mut u8 = 0xb8000 as *mut u8;

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    unsafe {
        let vga = 0xb8000 as *mut u64;

        *vga = 0x2f592f412f4b2f4f;
    };

    loop { }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
