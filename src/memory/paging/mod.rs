/*
 * 64-bit Paging Implementation
 * References:
 * https://wiki.osdev.org/Paging
 * https://www.youtube.com/watch?v=e47SApmmx44
 * https://www.udemy.com/course/developing-a-multithreaded-kernel-from-scratch
 */

extern crate volatile;
use crate::status::ErrorCode;
use bilge::prelude::*;
use core::convert::{TryFrom, TryInto};
use core::{arch::asm, mem::size_of};
use spin::Mutex;
use volatile::Volatile;
use KERNEL_HEAP;

/*
 * Each page table contains 512 8-byte entries
 */
const PAGING_TOTAL_ENTRIES_PER_TABLE: usize = 512;
const PAGING_PAGE_SIZE: usize = PAGING_TOTAL_ENTRIES_PER_TABLE * size_of::<usize>();

pub type PageAddress = *mut usize;

static CURRENT_PAGE_DIRECTORY: Mutex<Option<Paging256TBChunk>> = Mutex::new(None);

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
#[repr(C, packed)]
#[bitsize(64)]
#[derive(Clone, Copy, FromBits, Default)]
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
    no_execute: bool,
}

type PageDirectoryEntries = [Volatile<PageDirectoryEntry>; PAGING_TOTAL_ENTRIES_PER_TABLE];

#[repr(transparent)]
struct PageTable {
    entries: &'static mut PageDirectoryEntries,
}

impl PageTable {
    unsafe fn from(addr: PageAddress) -> Self {
        let entries = &mut *(addr as *mut PageDirectoryEntries);
        Self { entries }
    }

    unsafe fn from_pde(pde: PageDirectoryEntry) -> Self {
        let addr = u64::from(pde.addr() << 12) as PageAddress;
        Self::from(addr)
    }

    unsafe fn new() -> Result<Self, ErrorCode> {
        let addr = page_alloc(size_of::<PageDirectoryEntries>())?;
        Ok(Self::from(addr))
    }

    fn get_pt_or_insert(
        &mut self,
        idx: usize,
        flags: PageDirectoryEntry,
    ) -> Result<Self, ErrorCode> {
        let entry = self.entries.get_mut(idx).ok_or(ErrorCode::OutOfBounds)?;
        let mut pde = entry.read();

        if !pde.present() {
            let pt = unsafe { Self::new()? }; // not tracked by Rust's borrow checker
            pde = PageDirectoryEntry::from(pde.value | flags.value);
            pde.set_addr(u40::new(pt.entries.as_ptr() as u64) >> 12);
            pde.set_present(true);
            entry.write(pde);
        }

        Ok(unsafe { Self::from_pde(entry.read()) })
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
    p_i: usize,
}

impl PageMapIndexes {
    fn from(v_addr: PageAddress) -> Self {
        let mut v_addr_usize = v_addr as usize;
        v_addr_usize >>= 12;
        let p_i = v_addr_usize & 0x1ff;
        v_addr_usize >>= 9;
        let pt_i = v_addr_usize & 0x1ff;
        v_addr_usize >>= 9;
        let pd_i = v_addr_usize & 0x1ff;
        v_addr_usize >>= 9;
        let pdp_i = v_addr_usize & 0x1ff;
        Self {
            pdp_i,
            pd_i,
            pt_i,
            p_i,
        }
    }
}

/*
 * Manually allocate memory to prevent Rust from freeing pages at random
 */
unsafe fn page_alloc(size: usize) -> Result<PageAddress, ErrorCode> {
    Ok(KERNEL_HEAP.zalloc(size)? as PageAddress)
}

fn is_aligned(addr: PageAddress) -> bool {
    (addr as usize % PAGING_PAGE_SIZE) == 0
}

fn align_to_lower_page(addr: PageAddress) -> PageAddress {
    let mut addr_usize = addr as usize;
    addr_usize -= addr_usize % PAGING_PAGE_SIZE;
    addr_usize as PageAddress
}

pub struct Paging256TBChunk {
    plm4: PageTable,
}

impl Paging256TBChunk {
    /*
     * Memory must be manually freed since it's not tracked by rust's borrow checker
     */
    pub unsafe fn new() -> Result<Self, ErrorCode> {
        let plm4 = PageTable::new()?;

        let res = Self { plm4 };

        Ok(res)
    }

    pub fn set(
        &mut self,
        v_addr: PageAddress,
        val: u64,
        flags: PageDirectoryEntry,
    ) -> Result<(), ErrorCode> {
        if !is_aligned(v_addr) {
            return Err(ErrorCode::InvArg);
        }
        let idx = PageMapIndexes::from(v_addr);

        let mut plm3 = self.plm4.get_pt_or_insert(idx.pdp_i, flags)?;
        let mut plm2 = plm3.get_pt_or_insert(idx.pd_i, flags)?;
        let plm1 = plm2.get_pt_or_insert(idx.pt_i, flags)?;

        let mut pde = plm1
            .entries
            .get(idx.p_i)
            .ok_or(ErrorCode::OutOfBounds)?
            .read();

        pde = PageDirectoryEntry::from(pde.value | flags.value);
        pde.set_addr(u40::new(val >> 12));
        pde.set_present(true);

        plm1.entries
            .get_mut(idx.p_i)
            .ok_or(ErrorCode::OutOfBounds)?
            .write(pde);

        Ok(())
    }

    pub fn switch(new: Self) {
        let addr = new.plm4.entries.as_ptr();
        unsafe {
            asm! {
                "mov cr3, {0}",
                in(reg) addr
            }
        }

        let mut current_page_directory = CURRENT_PAGE_DIRECTORY.lock();
        *current_page_directory = Some(new);
    }

    pub fn map(
        &mut self,
        v_addr: PageAddress,
        p_addr: PageAddress,
        flags: PageDirectoryEntry,
    ) -> Result<(), ErrorCode> {
        if !is_aligned(v_addr) || !is_aligned(p_addr) {
            return Err(ErrorCode::InvArg);
        }
        self.set(v_addr, p_addr as u64, flags)
    }

    fn map_range(
        &mut self,
        mut v_addr: PageAddress,
        mut p_addr: PageAddress,
        count: usize,
        flags: PageDirectoryEntry,
    ) -> Result<(), ErrorCode> {
        for _ in 0..count {
            self.map(v_addr, p_addr, flags)?;
            // TODO test this
            v_addr = v_addr.wrapping_add(PAGING_PAGE_SIZE);
            p_addr = p_addr.wrapping_add(PAGING_PAGE_SIZE);
        }
        Ok(())
    }
    fn map_to(
        &mut self,
        v_addr: PageAddress,
        p_addr: PageAddress,
        p_addr_end: PageAddress,
        flags: PageDirectoryEntry,
    ) -> Result<(), ErrorCode> {
        if !is_aligned(v_addr) || !is_aligned(p_addr) || !is_aligned(p_addr_end) {
            return Err(ErrorCode::InvArg);
        }

        let p_addr_end_usize: usize = p_addr_end as usize;
        let p_addr_usize: usize = p_addr as usize;

        if p_addr_end_usize < p_addr_usize {
            return Err(ErrorCode::InvArg);
        }

        let total_bytes = p_addr_end_usize - p_addr_usize;
        let total_pages = total_bytes / PAGING_PAGE_SIZE;
        self.map_range(v_addr, p_addr, total_pages, flags)?;
        Ok(())
    }

    fn get_p_addr_or_insert(
        &mut self,
        v_addr: PageAddress,
        flags: PageDirectoryEntry,
    ) -> Result<PageAddress, ErrorCode> {
        let aligned_v_addr = align_to_lower_page(v_addr);

        let difference = (v_addr as usize).saturating_sub(aligned_v_addr as usize);

        let pde = self.get_or_insert(aligned_v_addr, flags)?;
        let pde_val_usize: usize = pde.value.try_into()?;
        Ok((pde_val_usize + difference) as PageAddress)
    }

    fn get_or_insert(
        &mut self,
        v_addr: PageAddress,
        flags: PageDirectoryEntry,
    ) -> Result<PageDirectoryEntry, ErrorCode> {
        let idx = PageMapIndexes::from(v_addr);

        let entry = self
            .plm4
            .get_pt_or_insert(idx.pdp_i, flags)?
            .get_pt_or_insert(idx.pd_i, flags)?
            .get_pt_or_insert(idx.pt_i, flags)?
            .entries
            .get(idx.p_i)
            .ok_or(ErrorCode::OutOfBounds)?
            .read();

        Ok(entry)
    }
}
