extern crate volatile;
extern crate spin;
use core::sync::atomic::AtomicPtr;

// 100MB Heap Definitions
// See https://wiki.osdev.org/Memory_Map_(x86) for more info 
pub const HEAP_SIZE_BYTES: usize = 104857600;
pub const HEAP_BLOCK_SIZE: usize = 4096;

pub const HEAP_ADDRESS: AtomicPtr<u8> = AtomicPtr::new(0x01000000 as *mut u8);
pub const HEAP_TABLE_ADDRESS: AtomicPtr<u8> = AtomicPtr::new(0x00007E00 as *mut u8);

pub const TOTAL_INTERRUPTS : usize = 512;
