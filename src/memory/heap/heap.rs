/**
 * Heap Implementation using First Fit Algorithm
 * References:
 * https://wiki.osdev.org/Interrupt_Descriptor_Table#Structure_on_x86-64
 */

extern crate volatile;
extern crate spin;
use bilge::arbitrary_int::u5;
use bilge::bitsize;
use bilge::prelude::Number;
use bilge::Bitsized;
use bilge::FromBits;
use core::convert::TryFrom;

use self::volatile::Volatile;
use core::alloc::{GlobalAlloc, Layout};
use core::convert::TryInto;
use core::sync::atomic::{AtomicPtr, Ordering}; // TODO is AtomicPtr necessary? If so, this needs to
                                               // be added to the paging implementation
use crate::status::ErrorCode;
use crate::config::{HEAP_ADDRESS, HEAP_BLOCK_SIZE, HEAP_SIZE_BYTES, HEAP_TABLE_ADDRESS};
use core::ptr;

/**
 * 8 Bit Entry Strucutre:
 * 0-3: Entry Type (Taken/Free)
 * 3-4: 0
 * 4-5: 0
 * 5-6: Is First
 * 6-7: Has Next
 */
#[repr(C)]
#[bitsize(8)]
#[derive(Clone, Copy, FromBits, Default)]
pub struct HeapBlockTableEntry {
    is_taken: bool,
    has_next: bool,
    is_first: bool,
    zero: u5
}

#[repr(transparent)]
struct HeapTable {
    // Each heap entry is always a multiple of HEAP_BLOCK_SIZE to not worry about paging
    entries: [Volatile<HeapBlockTableEntry>; HEAP_SIZE_BYTES / HEAP_BLOCK_SIZE]
}

/**
 * AtomicPtr needs to be used because the Heap doesn't have the Send trait, which would prevent the
 * Kernel Heap lazy_static! from working
 */
pub struct Heap {
    s_addr: AtomicPtr<u8>,
    table_addr: AtomicPtr<u8>
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

impl Heap {
    const fn new() -> Heap {

        let heap = Heap {
            s_addr: HEAP_ADDRESS,
            table_addr: HEAP_TABLE_ADDRESS
        };

        heap

    }

    pub fn init(&self) -> Result<(), ErrorCode> {

        let start = HEAP_ADDRESS;
        let end = AtomicPtr::new(unsafe { HEAP_ADDRESS.load(Ordering::Relaxed).add(HEAP_SIZE_BYTES) });

        if !heap_validate_alignment(&start) || !heap_validate_alignment(&end) {
            return Err(ErrorCode::EINVARG)
        }
        let total = HEAP_SIZE_BYTES / HEAP_BLOCK_SIZE;
        heap_validate_total_blocks(&start, &end, total)?;
        
        let table = self.get_table();

        // Ensure all blocks are marked free
        for entry in table.entries.iter_mut() {
            let entry_to_write = HeapBlockTableEntry::default();
            entry.write(entry_to_write);
        }

        Ok(())
    }

    fn block_to_address(&self, block: usize) -> *mut u8 {
        let _s_addr = self.s_addr.load(Ordering::Relaxed) as usize;
        return (_s_addr + (block * HEAP_BLOCK_SIZE)) as *mut u8;
    }

    fn address_to_block(&self, address: *mut u8) -> usize {
        let _s_addr = self.s_addr.load(Ordering::Relaxed) as usize;
        let _address = address as usize;
        return (_address - _s_addr) / HEAP_BLOCK_SIZE;
    }

    /**
     * Finds the first block s.t the blocks after it can fit total_blocks
     */
    fn get_start_block(&self, total_blocks: usize) -> Result<usize, ErrorCode> {
        let mut curr_block = 0;
        let mut start_block: isize = -1;
        let table = self.get_table();
        for i in 0..table.entries.len() {
            if table.entries[i].read().is_taken() {
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

    fn get_table(&self) -> &mut HeapTable {
        unsafe { &mut *(self.table_addr.load(Ordering::Relaxed) as *mut HeapTable) }
    }

    fn mark_blocks_taken(&self, start_block: usize, total_blocks: usize) -> () {
        let end_block = (start_block + total_blocks) - 1;

        let mut entry = HeapBlockTableEntry::default();
        entry.set_is_taken(true);
        entry.set_is_first(true);
        if total_blocks > 1 {
            entry.set_has_next(true);
        }

        let table = self.get_table();
        for i in start_block..end_block+1 {
            table.entries[i].write(entry);
            entry = HeapBlockTableEntry::default();
            entry.set_is_taken(true);
            if end_block > 0 && i != end_block - 1 {
                entry.set_has_next(true);
            }
        }
    }

    fn malloc_blocks(&self, total_blocks: usize) -> Result<*mut u8, ErrorCode> {
        let start_block = self.get_start_block(total_blocks)?;
        let address = self.block_to_address(start_block);
        self.mark_blocks_taken(start_block, total_blocks);
        return Ok(address);
    }

    fn mark_blocks_free(&self, starting_block: usize) -> () {
        let table = self.get_table();
        for i in starting_block..table.entries.len() {
            let entry = table.entries[i].clone();
            let mut entry_to_write = HeapBlockTableEntry::default();
            entry_to_write.set_is_taken(false);
            table.entries[i].write(entry_to_write);
            if !entry.read().has_next() {
                break;
            }
        }
    }

    pub fn malloc(&self, size: usize) -> Result<*mut u8, ErrorCode> {
        let aligned_size = heap_align_value_to_upper(size);
        let total_blocks = aligned_size / HEAP_BLOCK_SIZE;
        return self.malloc_blocks(total_blocks);
    }

    pub fn zalloc(&self, size: usize) -> Result<*mut u8, ErrorCode> {
        let ptr = self.malloc(size)?;

        unsafe {
            ptr::write_bytes(ptr, 0, size);
        }

        Ok(ptr)

    }

    pub fn free(&self, ptr: *mut u8) -> () {
        let block = self.address_to_block(ptr);
        self.mark_blocks_free(block);
    }
}

/*
 * Setup to use the heap as a global allocator
 */
unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        return self.malloc(layout.size()).unwrap();
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        return self.free(ptr);
    }
}

#[global_allocator]
pub static KERNEL_HEAP: Heap = Heap::new();
