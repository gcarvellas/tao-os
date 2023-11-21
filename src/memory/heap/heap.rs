// Rewritten heap implementation from https://github.com/nibblebits/PeachOS/tree/master/src/memory/heap

extern crate volatile;
use self::volatile::Volatile;
extern crate spin;
use core::convert::TryInto;
use crate::status::ErrorCode;
use crate::config::{HEAP_BLOCK_SIZE, HEAP_SIZE_BYTES};
use core::sync::atomic::{AtomicPtr, Ordering};
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
    s_addr: AtomicPtr<u8>,
    table: &'static mut HeapTable
}

fn heap_validate_alignment(ptr: &AtomicPtr<u8>) -> bool {
    let _ptr: usize = ptr.load(Ordering::Relaxed) as usize;
    return (_ptr % HEAP_BLOCK_SIZE) == 0;
}

fn heap_validate_total_blocks(start: &AtomicPtr<u8>, end: &AtomicPtr<u8>, total: usize) -> Result<(), ErrorCode> {
    let _start: usize = start.load(Ordering::Relaxed) as usize;
    let _end: usize = end.load(Ordering::Relaxed) as usize;
    let table_size = _end - _start;
    let total_blocks = table_size / HEAP_BLOCK_SIZE;
    if total != total_blocks {
        return Err(ErrorCode::EINVARG)
    }
    return Ok(())
}

fn heap_align_value_to_upper(mut val: usize) -> usize {
    if val % HEAP_BLOCK_SIZE == 0 {
        return val;
    }

    val = val - ( val % HEAP_BLOCK_SIZE);
    val += HEAP_BLOCK_SIZE;
    return val;
}

fn heap_get_entry_type(entry: &Volatile<HeapBlockTableEntry>) -> u8 {
    // Gets first 4 bits of table entry, which determines the type
    return entry.read() & 0x0f;
}

impl Heap {
    pub fn new(start: AtomicPtr<u8>, end: AtomicPtr<u8>, total: usize, table_address: AtomicPtr<u8> ) -> Result<Heap, ErrorCode> {
        if !heap_validate_alignment(&start) || !heap_validate_alignment(&end) {
            return Err(ErrorCode::EINVARG)
        }
        heap_validate_total_blocks(&start, &end, total).ok();
        let _table = unsafe { &mut *(table_address.load(Ordering::Relaxed) as *mut HeapTable) };

        for entry in _table.entries.iter_mut() {
            entry.write(HEAP_BLOCK_TABLE_ENTRY_FREE);
        }

        return Ok(Heap { s_addr: start, table: _table })

    }

    fn block_to_address(&mut self, block: usize) -> *mut u8 {
        let _s_addr = self.s_addr.load(Ordering::Relaxed) as usize;
        return (_s_addr + (block * HEAP_BLOCK_SIZE)) as *mut u8;
    }

    fn address_to_block(&mut self, address: *mut u8) -> usize {
        let _s_addr = self.s_addr.load(Ordering::Relaxed) as usize;
        let _address = address as usize;
        return (_address - _s_addr) / HEAP_BLOCK_SIZE;
    }

    fn get_start_block(&mut self, total_blocks: usize) -> Result<usize, ErrorCode> {
        let mut curr_block = 0;
        let mut start_block: isize = -1;
        for i in 0..self.table.entries.len() {
            if heap_get_entry_type(&self.table.entries[i]) != HEAP_BLOCK_TABLE_ENTRY_FREE {
                curr_block = 0;
                start_block = -1;
                continue;
            }

            if start_block == -1 {
                start_block = i.try_into().unwrap();
            }

            curr_block+=1;

            if curr_block == total_blocks {
                break;
            }
        }

        if start_block == -1 {
            return Err(ErrorCode::ENOMEM);
        }
        return Ok(start_block.try_into().unwrap());
    }

    fn mark_blocks_taken(&mut self, start_block: usize, total_blocks: usize) -> () {
        let end_block = (start_block + total_blocks) - 1;

        let mut entry = HEAP_BLOCK_TABLE_ENTRY_TAKEN | HEAP_BLOCK_IS_FIRST;
        if total_blocks > 1 {
            entry |= HEAP_BLOCK_HAS_NEXT;
        }
        for i in start_block..end_block+1 {
            self.table.entries[i].write(entry);
            entry = HEAP_BLOCK_TABLE_ENTRY_TAKEN;
            if end_block > 0 && i != end_block - 1 {
                entry |= HEAP_BLOCK_HAS_NEXT;
            }
        }
    }

    fn malloc_blocks(&mut self, total_blocks: usize) -> Result<*mut u8, ErrorCode> {
        let start_block = self.get_start_block(total_blocks).unwrap();
        let address = self.block_to_address(start_block);
        self.mark_blocks_taken(start_block, total_blocks);
        return Ok(address);
    }

    fn mark_blocks_free(&mut self, starting_block: usize) -> () {
        for i in starting_block..self.table.entries.len() {
            let entry = self.table.entries[i].clone();
            self.table.entries[i].write(HEAP_BLOCK_TABLE_ENTRY_FREE);
            if entry.read() & HEAP_BLOCK_HAS_NEXT == 0 {
                break;
            }
        }
    }

    pub fn malloc(&mut self, size: usize) -> Result<*mut u8, ErrorCode> {
        let aligned_size = heap_align_value_to_upper(size);
        let total_blocks = aligned_size / HEAP_BLOCK_SIZE;
        return self.malloc_blocks(total_blocks);
    }
    pub fn free(&mut self, ptr: *mut u8) -> () {
        let block = self.address_to_block(ptr);
        self.mark_blocks_free(block);
    }
}
