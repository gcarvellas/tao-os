// Rewritten heap implementation from https://github.com/nibblebits/PeachOS/tree/master/src/memory/heap

extern crate volatile;
use self::volatile::Volatile;

use crate::{status::ErrorCode, config::{HEAP_BLOCK_SIZE, HEAP_SIZE_BYTES}};

const HEAP_BLOCK_TABLE_ENTRY_TAKEN: u8 = 0x01;
const HEAP_BLOCK_TABLE_ENTRY_FREE: u8 = 0x00;

// Heap Table Entry Definition
const HEAP_BLOCK_HAS_NEXT: u8 = 0b10000000;
const HEAP_BLOCK_IS_FIRST: u8 = 0b01000000;

pub type HeapBlockTableEntry = u8;

#[repr(transparent)]
struct HeapTable {
    entries: [Volatile<HeapBlockTableEntry>; HEAP_SIZE_BYTES / HEAP_BLOCK_SIZE]
}

pub struct Heap {
    s_addr: usize,
    table: &'static mut HeapTable
}

fn heap_validate_alignment(ptr: usize) -> bool {
    return ((ptr as usize) % HEAP_BLOCK_SIZE) == 0
}

fn heap_validate_total_blocks(start: usize, end: usize, total: usize) -> Result<(), ErrorCode> {
    let table_size = (end as usize - start as usize) as usize;
    let total_blocks = table_size / HEAP_BLOCK_SIZE;
    if total != total_blocks {
        return Err(ErrorCode::EINVARG)
    }
    return Ok(())
}

impl Heap {
    pub fn new(start: usize, end: usize, total: usize, table_address: usize) -> Result<Heap, ErrorCode> {
        if !heap_validate_alignment(start) || !heap_validate_alignment(end) {
            return Err(ErrorCode::EINVARG)
        }
        heap_validate_total_blocks(start, end, total).ok();
        let _table = unsafe { &mut *(table_address as *mut HeapTable) };

        for entry in _table.entries.iter_mut() {
            entry.write(HEAP_BLOCK_TABLE_ENTRY_FREE);
        }

        return Ok(Heap { s_addr: start, table: _table })

    }
    pub fn heap_malloc(&mut self, size: usize) -> Result<*mut u8, ErrorCode> {
        unimplemented!();
    }
    pub fn heap_free(&mut self, ptr: *mut u8) -> Result<(), ErrorCode> {
        unimplemented!();
    }
}
