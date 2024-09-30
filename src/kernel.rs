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
extern crate bilge;
extern crate volatile;
use crate::idt::idt::Idt;
use crate::idt::idt::disable_interrupts;
use crate::idt::idt::enable_interrupts;
use crate::memory::paging::paging::PageAddress;
use crate::io::vga::VgaDisplay;
use crate::memory::paging::paging::PageDirectoryEntry;
use crate::memory::paging::paging::Paging256TBChunk;
use core::panic::PanicInfo;
use alloc::boxed::Box;
use lazy_static::lazy_static;
use memory::heap::kheap::KernelHeap;
use spin::Mutex;

lazy_static! {
    static ref SCREEN: Mutex<VgaDisplay> = Mutex::new(VgaDisplay::default());
    static ref IDT: Idt = Idt::default();
    static ref KERNEL_HEAP: Mutex<KernelHeap> = Mutex::new(KernelHeap::default().unwrap());
    static ref CURRENT_PAGE_DIRECTORY: Mutex<Option<Paging256TBChunk<'static>>> = Mutex::new(None);
}

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    disable_interrupts();
    println!("Kernel Panic! :( \n");
    let args = panic_info.message();
    println!("Message: {}", args);

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
    {
        let tmp = Box::new(42);
        println!("This is on the heap: {}.", tmp);
    }
}

fn test_paging() -> () {
    println!("Creating a new paging chunk");
    let mut flags = PageDirectoryEntry::default();
    flags.set_writeable(true);
    flags.set_present(true);
    flags.set_access_from_all(true);
    let mut chunk = Paging256TBChunk::new().unwrap();

    let ptr = Box::new("No");
    for i in 0..51200 { // 512*512*4
        let address = i * 0x1000;
        if address == 0x1000 {
            chunk.set(address as PageAddress, (ptr.as_ptr() as u64 | 0x7) as u64, flags).unwrap();
        } else {
            chunk.set(address as PageAddress, address, flags).unwrap();
        }
    }

                                                             
    // TODO once chunk.switch() is called, the main kernel page is lost
    chunk.switch();
    println!("After switch");

    let ptr2 = 0x1000 as *mut char;
    unsafe {
        *ptr2 = 'A';
        *(ptr2.offset(1)) = 'B';
    }
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
