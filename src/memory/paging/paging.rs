use crate::status::ErrorCode;
use crate::lazy_static::lazy_static;
use core::arch::asm;
use alloc::boxed::Box;
use spin::Mutex;
use core::mem::size_of;

/**
 *  x86_64 PDE Layout:
 *  0-1: Present
 *  1-2: Writeable
 *  2-3: Access from user and kernel space
 *  3-4: Write Through
 *  4-5: Cache Disabled
 *  5-6: Accessed
 *  6-7: Dirty
 *  7-8: Page Attribute Table
 *  8-9: Global
 *  9-11: Available
 *  12-50: Address bits
 *  50-51: Reserved (0)
 *  52-58: Available
 *  59-62: Protection Key
 *  62-63: Execute Disabled
 *  See 64-Bit Paging https://wiki.osdev.org/Paging
 */
pub const PDE_PRESENT: usize = 1 << 0;
pub const PDE_WRITEABLE: usize = 1 << 1;
pub const PDE_ACCESS_FROM_ALL: usize = 1 << 2;
pub const PDE_WRITE_THROUGH: usize = 1 << 3;
pub const PDE_CACHE_DISABLED: usize = 1 << 4;
pub const PDE_LARGE: usize = 1 << 7;

/*
 * Each page table contains 512 8-byte entries
 */
const PAGING_TOTAL_ENTRIES_PER_TABLE: usize = 512;
const PAGING_PAGE_SIZE: usize = PAGING_TOTAL_ENTRIES_PER_TABLE * size_of::<usize>();

type PageTable = Box<[usize; PAGING_TOTAL_ENTRIES_PER_TABLE]>; 

lazy_static! {
    static ref CURRENT_DIRECTORY: Mutex<usize> = Mutex::new(0);
}

struct PagingIndexes {
    directory: *mut usize,
    table: *mut usize
}

#[inline(always)]
fn new_page_table() -> PageTable {
    return Box::new([0; PAGING_TOTAL_ENTRIES_PER_TABLE]);
}

#[inline(always)]
fn paging_is_aligned(addr: *mut usize) -> bool {
    return (addr as usize % PAGING_PAGE_SIZE) == 0;
}

fn paging_get_indexes(virtual_address: *mut usize) -> Result<PagingIndexes, ErrorCode> {
    if !paging_is_aligned(virtual_address) {
        return Err(ErrorCode::EINVARG);
    }
    // TODO math is wrong since it assumes 32 bit paging
    return Ok(PagingIndexes {
        directory: ((virtual_address as usize) / PAGING_TOTAL_ENTRIES_PER_TABLE * PAGING_PAGE_SIZE) as *mut usize,
        table: (((virtual_address as usize) / PAGING_TOTAL_ENTRIES_PER_TABLE * PAGING_PAGE_SIZE) / PAGING_PAGE_SIZE) as *mut usize
    });
}

fn paging_align_address(ptr: *mut usize) -> *mut usize {
    let _ptr = ptr as usize;
    if _ptr % PAGING_PAGE_SIZE != 0 {
        return (_ptr + PAGING_PAGE_SIZE - (_ptr % PAGING_PAGE_SIZE)) as *mut usize;
    }
    return ptr;
}

fn paging_align_to_lower_page(addr: *mut usize) -> *mut usize {
    let mut _addr = addr as usize;
    _addr -= _addr % PAGING_PAGE_SIZE;
    return _addr as *mut usize;
}

pub struct Paging256TBChunk {
    directory_entry: PageTable
}

/**
 * x86_64: 256TB Chunk contains the plm4_table, plm3_table, 
 * and plm2_table
 */

// TODO if Rust is trying to free the current page, try to switch to another available chunk. If
// not, panic.
impl Paging256TBChunk {
    pub fn new(flags: usize) -> Paging256TBChunk {
        let mut plm4_table = new_page_table(); 
        let mut offset = 0;

        for i in 0..plm4_table.len() {
            let mut plm3_table = new_page_table();
            for j in 0..plm3_table.len() {
                let mut plm2_table = new_page_table();
                for k in 0..plm2_table.len() {
                    plm2_table[k] = (offset + (k * PAGING_PAGE_SIZE)) | flags | PDE_LARGE; 
                }
                offset += PAGING_TOTAL_ENTRIES_PER_TABLE * PAGING_PAGE_SIZE;
                plm3_table[j] = (*plm2_table).as_ptr() as usize | flags | PDE_WRITEABLE; 
            }
            plm4_table[i] = (*plm3_table).as_ptr() as usize | flags | PDE_WRITEABLE; 
        }
        return Paging256TBChunk { directory_entry: plm4_table };
    }
    pub fn switch(&self) -> () {
        unsafe {
            asm! {
                "mov cr3, {0}",
                in(reg) &*self.directory_entry 
            }
        }
        let mut current_directory = CURRENT_DIRECTORY.lock();
        *current_directory = (*self.directory_entry).as_ptr() as usize
    }
    fn map(&self, virt: *mut usize, phys: *mut usize, flags: usize) -> Result<(), ErrorCode>{
        let _virt = virt as usize;
        let _phys = phys as usize;
        if _virt % PAGING_PAGE_SIZE == 0 || _phys % PAGING_PAGE_SIZE == 0 {
            return Err(ErrorCode::EINVARG);
        }
        return self.set(virt, (phys as usize) | flags as usize);
    }
    fn map_range(&self, virt: *mut usize, phys: *mut usize, count: usize, flags: usize) -> Result<(), ErrorCode>{
        unimplemented!();
    }
    fn map_to(&self, virt: *mut usize, phys: *mut usize, phys_end: *mut u8, flags: usize) -> Result<(), ErrorCode>{
        unimplemented!();
    }
    fn get_physical_address(&self, virt: *mut usize) -> *mut usize {
        unimplemented!();
    }
    fn get(&self, virt: *mut usize) -> *mut usize {
        unimplemented!();
    }
    fn set(&self, virt: *mut usize, val: usize) -> Result<(), ErrorCode> {
        // TODO this assumes a 32bit paging setup. Doesn't work
        if !paging_is_aligned(virt) {
            return Err(ErrorCode::EINVARG);
        }
        let indexes = paging_get_indexes(virt).unwrap();

        // TODO we have a reference to the actual array. Use that instead.
        let entry = self.directory_entry[indexes.directory as usize];
        let table_ptr = (entry & 0xfffff000) as *mut usize;
        let table_index = indexes.table as usize;
        unsafe {
            table_ptr.add(table_index).write(val);
        }
        return Ok(());
    }
}
