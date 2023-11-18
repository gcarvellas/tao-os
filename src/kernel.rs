#![no_std]
#![no_main]

mod io;
mod status;

use core::fmt::Write;
use crate::io::vga::VgaDisplay;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {

    let mut display = VgaDisplay::new();

    // TODO clear display
    
    display.write_char('H');
    display.write_char('e');
    display.write_char('l');
    display.write_char('l');
    display.write_char('o');
    display.write_char('!');

    // For now, this should panic since write_fmt is not implemented
    //write_str!(display, "Test!");

    loop { }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        let vga = 0xb8020 as *mut u64;

        *vga = 0x2f592f412f4b2f4f; // prints TEST
    };

    loop {}
}
