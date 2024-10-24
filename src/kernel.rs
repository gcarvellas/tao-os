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
#![deny(clippy::nonminimal_bool)]
#![allow(clippy::mut_from_ref)] // Requried for Heap get_table

// Required for AtomicPtr
#![warn(clippy::declare_interior_mutable_const)]
#![warn(clippy::borrow_interior_mutable_const)]

mod config;
mod disk;
mod fs;
mod idt;
mod io;
mod memory;
mod status;
extern crate alloc;
extern crate bilge;
extern crate hashbrown;
extern crate spin;
extern crate volatile;
use crate::idt::disable_interrupts;
use crate::idt::enable_interrupts;
use crate::memory::heap::KERNEL_HEAP;
use crate::memory::paging::PageAddress;
use crate::memory::paging::PageDirectoryEntry;
use crate::memory::paging::Paging256TBChunk;
use alloc::boxed::Box;
use alloc::string::String;
use core::panic::PanicInfo;
use fs::file::fclose;
use fs::file::fopen;
use fs::file::fread;
use fs::file::fstat;
use idt::IDT;

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
    let mut chunk = unsafe { Paging256TBChunk::new().unwrap() }; // page is not freed

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
    Paging256TBChunk::switch(chunk);
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

// TODO use cargo's testing to do this https://os.phil-opp.com/testing/
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

    println!("Attempting to open 1:/HELLO.TXT");
    let fd = fopen("1:/HELLO.TXT", "r").expect("Failed to open HELLO.TXT");
    println!("We opened 1:/HELLO.TXT at fd {}", fd);

    let mut buf = [0; 8];
    fread(&mut buf, 8, 1, fd).expect("Failed to read HELLO.TXT");
    let result = String::from_utf8(buf.to_vec());
    println!("We read 1:/HELLO.TXT: \"{:?}\"", result);

    let stats = fstat(fd).expect("failed to stat HELLO.TXT");
    println!("stats: {:?}", stats);
    fclose(fd).expect("Failed to close HELLO.TXT");

    println!("Testing a kernel panic using Rust's unimplemented! macro.");

    unimplemented!();

    //loop { }
}
