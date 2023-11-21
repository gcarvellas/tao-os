#![feature(panic_info_message)]

#![no_std]
#![no_main]

mod io;
mod memory;
mod config;
mod status;
extern crate lazy_static;
extern crate spin;
extern crate alloc;
use crate::io::vga::VgaDisplay;
use core::panic::PanicInfo;
use alloc::boxed::Box;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref SCREEN: Mutex<VgaDisplay> = Mutex::new(VgaDisplay::default());
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

    println!("This is currently using the rust println! macro. {}", "Hello World");

    let tmp = Box::new(42);
    println!("This is on the heap: {}", tmp);
    
    unimplemented!();

    loop { }
}
