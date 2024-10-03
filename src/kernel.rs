#![no_std]
#![no_main]
#![deny(clippy::cast_lossless)]
#![deny(clippy::cast_possible_truncation)]
#![deny(clippy::cast_precision_loss)]
#![deny(clippy::cast_sign_loss)]
#![deny(clippy::cast_possible_wrap)]
#![deny(clippy::clone_on_ref_ptr)]
#![deny(clippy::default_trait_access)]
#![deny(clippy::doc_markdown)]
#![deny(clippy::fallible_impl_from)]
#![deny(clippy::indexing_slicing)]
#![deny(clippy::linkedlist)]
#![deny(clippy::match_same_arms)]
#![deny(clippy::maybe_infinite_iter)]
#![deny(clippy::mem_forget)]
#![deny(clippy::multiple_inherent_impl)]
#![deny(clippy::mut_mut)]
#![deny(clippy::mutex_integer)]
#![deny(clippy::needless_borrow)]
#![deny(clippy::needless_continue)]
#![deny(clippy::map_unwrap_or)]
#![deny(clippy::unwrap_in_result)]
#![deny(clippy::unwrap_or_default)]
#![deny(clippy::panicking_unwrap)]
#![deny(clippy::range_plus_one)]
#![deny(clippy::single_match_else)]
#![deny(clippy::string_add)]
#![deny(clippy::string_add_assign)]
#![deny(clippy::unnecessary_unwrap)]
#![deny(clippy::unseparated_literal_suffix)]
#![deny(clippy::use_self)]
#![deny(clippy::used_underscore_binding)]
#![deny(clippy::needless_range_loop)]
#![deny(clippy::declare_interior_mutable_const)]
#![allow(clippy::declare_interior_mutable_const)] // Required for AtomicPtr
#![allow(clippy::mut_from_ref)] // Requried for Heap get_table

mod config;
mod idt;
mod io;
mod memory;
mod status;
extern crate alloc;
extern crate bilge;
extern crate lazy_static;
extern crate spin;
extern crate volatile;
use crate::idt::idt::disable_interrupts;
use crate::idt::idt::enable_interrupts;
use crate::idt::idt::Idt;
use crate::io::vga::VgaDisplay;
use crate::memory::heap::KERNEL_HEAP;
use crate::memory::paging::paging::PageAddress;
use crate::memory::paging::paging::PageDirectoryEntry;
use crate::memory::paging::paging::Paging256TBChunk;
use alloc::boxed::Box;
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref SCREEN: Mutex<VgaDisplay> =
        Mutex::new(VgaDisplay::new().expect("Failed to initialize VGA"));
    static ref IDT: Idt = Idt::new().expect("Failed to initialize IDT");
    static ref CURRENT_PAGE_DIRECTORY: Mutex<Option<Paging256TBChunk<'static>>> = Mutex::new(None);
}

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    disable_interrupts();
    println!("Kernel Panic! :( \n");
    let args = panic_info.message();
    println!("Message: {}", args);

    if let Some(location) = panic_info.location() {
        println!(
            "Location: Panic occurred in file '{}' at line {}",
            location.file(),
            location.line()
        );
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

fn test_malloc() {
    {
        let tmp = Box::new(42);
        println!("This is on the heap: {}.", tmp);
    }
}

fn test_paging() {
    println!("Creating a new paging chunk");
    let mut flags = PageDirectoryEntry::default();
    flags.set_writeable(true);
    flags.set_present(true);
    flags.set_access_from_all(true);
    let mut chunk = Paging256TBChunk::new().unwrap();

    let ptr = Box::new("No");
    for i in 0..51200 {
        // 512*512*4
        let address = i * 0x1000;
        if address == 0x1000 {
            chunk
                .set(address as PageAddress, ptr.as_ptr() as u64 | 0x7, flags)
                .unwrap();
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
    KERNEL_HEAP.init().unwrap();

    IDT.load();
    enable_interrupts();

    println!(
        "This is currently using the rust println! macro. {}",
        "Hello World"
    );

    test_malloc();
    println!("Successfully deallocated memory.");

    test_paging();

    println!("Testing a kernel panic using Rust's unimplemented! macro.");

    unimplemented!();

    //loop { }
}
