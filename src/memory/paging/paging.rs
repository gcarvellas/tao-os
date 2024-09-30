/*
 * 64-bit Paging Implementation
 * References: 
 * https://wiki.osdev.org/Paging
 * https://www.youtube.com/watch?v=e47SApmmx44
 * https://www.udemy.com/course/developing-a-multithreaded-kernel-from-scratch
 */

extern crate volatile;
use core::{arch::asm, mem::size_of};
use bilge::prelude::*;
use volatile::Volatile;
use core::convert::TryFrom;
use crate::{status::ErrorCode, KERNEL_HEAP};

/*
 * Each page table contains 512 8-byte entries
 */
const PAGING_TOTAL_ENTRIES_PER_TABLE: usize = 512;
const PAGING_PAGE_SIZE: usize = PAGING_TOTAL_ENTRIES_PER_TABLE * size_of::<usize>();

pub type PageAddress = *mut usize;

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
 */
#[repr(C)]
#[bitsize(64)]
#[derive(Clone, Copy, FromBits)]
pub struct PageDirectoryEntry {
    pub present: bool,
    pub writeable: bool,
    pub access_from_all: bool,
    write_through_caching: bool,
    disable_cache: bool,
    accessed: bool,
    dirty: bool,
    huge_page: bool,
    global: bool,
    available_low: u3,
    addr: u40,
    available_high: u11,
    no_execute: bool
}

impl PageDirectoryEntry {

    pub fn allocate_new(flags: PageDirectoryEntry) -> Result<PageDirectoryEntry, ErrorCode> {
        let addr = page_alloc(size_of::<PageDirectoryEntry>())?;
        Ok(PageDirectoryEntry::from((addr as u64) | flags.value))
    }

    pub fn default() -> PageDirectoryEntry {
        PageDirectoryEntry::from(0x00)
    }

}

type PageDirectoryEntries = [Volatile<PageDirectoryEntry>; PAGING_TOTAL_ENTRIES_PER_TABLE];

#[repr(transparent)]
struct PageTable<'a> {
    entries: &'a mut PageDirectoryEntries
} 

impl<'a> PageTable<'a> {

    unsafe fn from(addr: PageAddress) -> PageTable<'a> {
        let entries = unsafe { &mut *(addr as *mut PageDirectoryEntries) };
        PageTable {
            entries
        }
    }

    unsafe fn from_pde(pde: PageDirectoryEntry) -> PageTable<'a> {
        let addr = u64::from(pde.addr() << 12) as PageAddress; 
        PageTable::from(addr)
    }

    fn new() -> Result<PageTable<'a>, ErrorCode> {
        let addr = page_alloc(size_of::<PageDirectoryEntries>())?;
        unsafe { Ok(PageTable::from(addr)) }
    }

    fn get_pt(&mut self, idx: usize, flags: PageDirectoryEntry) -> Result<PageTable, ErrorCode> {
        let mut pde = self.entries[idx].read();
        if !pde.present() {
            let pt = PageTable::new()?;
            pde = PageDirectoryEntry::from(pde.value | flags.value);
            pde.set_addr(u40::new(pt.entries.as_ptr() as u64) >> 12);
            pde.set_present(true);
            self.entries[idx].write(pde);
        }
        Ok(unsafe { PageTable::from_pde(self.entries[idx].read()) })
    }
}

/*
 * PLM4 Table contains 512 PDP Tables. A PDP Table contains 512 PD Tables. a PD Table contains 512
 * P tables. a P table contains 512 Pages
 */
struct PageMapIndexes {
    pdp_i: usize,
    pd_i: usize,
    pt_i: usize,
    p_i: usize
}

impl PageMapIndexes {
    fn from(_v_addr: PageAddress) -> PageMapIndexes {
        let mut v_addr = _v_addr as usize;
        v_addr>>=12;
        let p_i = v_addr & 0x1ff;
        v_addr >>= 9;
        let pt_i = v_addr & 0x1ff;
        v_addr >>= 9;
        let pd_i = v_addr & 0x1ff;
        v_addr >>= 9;
        let pdp_i = v_addr & 0x1ff;
        PageMapIndexes {
            pdp_i,
            pd_i,
            pt_i,
            p_i
        }
    }
}

/*
 * Manually allocate memory to prevent Rust from freeing pages at random
 */
fn page_alloc(size: usize) -> Result<PageAddress, ErrorCode> {
    Ok(KERNEL_HEAP.lock().heap.zalloc(size)? as PageAddress)
}

#[inline(always)]
fn paging_is_aligned(addr: PageAddress) -> bool {
    return (addr as usize % PAGING_PAGE_SIZE) == 0;
} 

pub struct Paging256TBChunk<'a> {
    plm4: PageTable<'a>,
}

impl<'a> Paging256TBChunk<'a> {

    pub fn new() -> Result<Paging256TBChunk<'a>, ErrorCode> {
        let plm4 = PageTable::new()?;  

        let res = Paging256TBChunk {
            plm4,
        };

        Ok(res)
    }

    pub fn set(&mut self, virt: PageAddress, val: u64, flags: PageDirectoryEntry) -> Result<(), ErrorCode> {
        if !paging_is_aligned(virt) {
            return Err(ErrorCode::EINVARG);
        }
        let idx = PageMapIndexes::from(virt);

        let mut plm3 = self.plm4.get_pt(idx.pdp_i, flags)?; 
        let mut plm2 = plm3.get_pt(idx.pd_i, flags)?; 
        let plm1 = plm2.get_pt(idx.pt_i, flags)?; 

        let mut pde = plm1.entries[idx.p_i].read();

        pde = PageDirectoryEntry::from(pde.value | flags.value);
        pde.set_addr(u40::new(val >> 12));
        pde.set_present(true);

        plm1.entries[idx.p_i].write(pde);

        Ok(())
    }

    pub fn switch(&self) -> () {
        let addr = self.plm4.entries.as_ptr();
        unsafe {
            asm! {
                "mov cr3, {0}",
                in(reg) addr
            }
            // TODO update CURRENT_PAGE_DIRECTORY
        }
    }

    pub fn map(&mut self, virt: PageAddress, phys: PageAddress, flags: PageDirectoryEntry) -> Result<(), ErrorCode>{
        if !paging_is_aligned(virt) || !paging_is_aligned(phys) {
            return Err(ErrorCode::EINVARG);
        }
        return self.set(virt, phys as u64, flags);
    }

    fn map_range(&self, virt: PageAddress, phys: PageAddress, count: usize, flags: PageDirectoryEntry) -> Result<(), ErrorCode>{
        unimplemented!();
    }
    fn map_to(&self, virt: PageAddress, phys: PageAddress, phys_end: *mut u8, flags: PageDirectoryEntry) -> Result<(), ErrorCode>{
        unimplemented!();
    }
    fn get_p_addr(&self, virt: PageAddress) -> *mut usize {
        unimplemented!();
    }
    fn get(&self, virt: PageAddress) -> *mut usize {
        unimplemented!();
    }

}

