#![no_std]
#![no_main]
// Clippy
#![deny(clippy::cast_lossless)]
#![deny(clippy::cast_possible_truncation)]
#![deny(clippy::cast_precision_loss)]
#![deny(clippy::cast_sign_loss)]
#![deny(clippy::cast_possible_wrap)]
#![deny(clippy::clone_on_ref_ptr)]
#![deny(clippy::default_trait_access)]
#![deny(clippy::doc_markdown)]
#![deny(clippy::fallible_impl_from)]
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
#![deny(clippy::manual_assert)]
#![deny(clippy::missing_asserts_for_indexing)]
#![deny(clippy::undocumented_ansafe_blocks)]
/*
 * Clippy: required for Heap get_table
 * TODO: Fix this if possible
 */
#![allow(clippy::mut_from_ref)]
/*
 * Clippy: required for AtomicPtr
 */
#![warn(clippy::declare_interior_mutable_const)]
#![warn(clippy::borrow_interior_mutable_const)]

mod arch;
mod config;
mod disk;
mod fs;
mod io;
mod memory;
mod status;

#[cfg(feature = "integration")]
mod tests;

extern crate alloc;
extern crate bilge;
extern crate hashbrown;
extern crate spin;
extern crate volatile;

#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::{
    idt::{disable_interrupts, enable_interrupts, IDT},
    io::isr::hault,
};

use crate::memory::heap::KERNEL_HEAP;
use core::panic::PanicInfo;

#[cfg(not(feature = "integration"))]
fn on_panic() -> ! {
    hault();
}

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    // Safety: Stop all hardware asap during a panic
    unsafe {
        disable_interrupts();
    }
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

    #[cfg(not(feature = "integration"))]
    on_panic();

    #[cfg(feature = "integration")]
    {
        use tests::on_panic;
        on_panic();
    }
}

#[cfg(feature = "integration")]
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    use tests::test_main;

    test_main();
}

pub fn kernel_init() {
    KERNEL_HEAP
        .init()
        .expect("Failed to initialize kernel heap");

    IDT.load();
    // Safety: initializers above will properly handle interrupts
    unsafe { enable_interrupts() };
}

#[cfg(not(feature = "integration"))]
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    kernel_init();
    unimplemented!();
}
