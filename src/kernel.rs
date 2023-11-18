#![feature(panic_info_message)]

#![no_std]
#![no_main]

mod io;
extern crate lazy_static;
extern crate spin;
use crate::io::vga::VgaDisplay;
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref SCREEN: Mutex<VgaDisplay> = Mutex::new(VgaDisplay::new());
}

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    if let Some(args) = panic_info.message() {
        println!("Kernel Panic! {}", args);
    } else {
        println!("Kernel Panic! Unknown panic message.");
    }
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {

    println!("I'm using the print macro rn!{}", "test");

    unimplemented!();
    loop { }
}
