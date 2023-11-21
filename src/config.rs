// 100MB Heap Definitions
// See https://wiki.osdev.org/Memory_Map_(x86) for more info 
pub const HEAP_SIZE_BYTES: usize = 104857600;
pub const HEAP_BLOCK_SIZE:  usize = 4096;

pub const HEAP_ADDRESS: usize  = 0x01000000 as usize;
pub const HEAP_TABLE_ADDRESS: usize = 0x00007E00;
pub const SECTOR_SIZE: usize = 512;
