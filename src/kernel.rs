// Required for panic handler
#![feature(panic_info_message)]

#![no_std]
#![no_main]

mod io;
mod memory;
mod config;
mod status;
mod idt;
extern crate lazy_static;
extern crate spin;
extern crate alloc;
use crate::idt::idt::Idt;
use crate::idt::idt::disable_interrupts;
use crate::idt::idt::enable_interrupts;
use crate::memory::paging::paging::PDE_ACCESS_FROM_ALL;
use crate::memory::paging::paging::PDE_PRESENT;
use crate::memory::paging::paging::PDE_WRITEABLE;
use crate::memory::paging::paging::Paging256TBChunk;
use crate::io::vga::VgaDisplay;
use core::panic::PanicInfo;
use alloc::boxed::Box;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref SCREEN: Mutex<VgaDisplay> = Mutex::new(VgaDisplay::default());
    static ref IDT: Idt = Idt::default();
}

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    disable_interrupts();
    println!("Kernel Panic! :( \n");
    if let Some(args) = panic_info.message() {
        println!("Message: {}", args);
    } else {
        println!("Message: Unknown");
    }

    if let Some(location) = panic_info.location() {
        println!("Location: Panic occurred in file '{}' at line {}", location.file(), location.line());
    } else {
        println!("Location: Unknown");
    }

    if let Some(payload) = panic_info.payload().downcast_ref::<&str>() {
        println!("Payload: {}", payload);
    } else {
        println!("Payload: Unknown");
    }

    loop {}
}

fn test_malloc() -> () {
    let tmp = Box::new(42);
    println!("This is on the heap: {}.", tmp);
}

fn test_paging() -> () {
    println!("Creating a new paging chunk");
    let chunk = Paging256TBChunk::new(PDE_WRITEABLE | PDE_PRESENT | PDE_ACCESS_FROM_ALL);

    let ptr = Box::new("No");
    chunk.set(0x1000 as *mut usize, (ptr.as_ptr() as usize) | PDE_ACCESS_FROM_ALL | PDE_WRITEABLE | PDE_PRESENT);

    // TODO once chunk.switch() is called, the main kernel page is lost
    chunk.switch();
    println!("After switch");

    let ptr2 = 0x1000 as *mut char;
    unsafe {
        *ptr2 = 'A';
        *(ptr2.offset(1)) = 'B';
    }
    let mut index = 0;
    unsafe {
        let c1 = core::ptr::read(ptr2);
        let c2 = core::ptr::read(ptr2.add(1));
        assert!(c1 == *ptr2);
        assert!(c2 == *(ptr2.offset(1)));
    }
    println!("Paging works!");
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {

    IDT.load();
    enable_interrupts();

    println!("This is currently using the rust println! macro. {}", "Hello World");
   
    test_malloc();
    println!("Successfully deallocated memory.");

    test_paging();

    println!("Testing a kernel panic using Rust's unimplemented! macro.");
    unimplemented!();

    loop { }
}
