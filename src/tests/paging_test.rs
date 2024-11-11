use crate::memory::paging::PageAddress;
use crate::memory::paging::PageDirectoryEntry;
use crate::memory::paging::Paging256TBChunk;
use crate::println;
use crate::status::ErrorCode;
use alloc::boxed::Box;
use alloc::format;

macro_rules! log {
    ($($arg:tt)*) => {
        println!("[paging_test] {}", format!($($arg)*));
    };
}

pub fn paging_test() -> Result<(), ErrorCode> {
    log!("Creating a page...");
    let mut flags = PageDirectoryEntry::default();
    flags.set_writeable(true);
    flags.set_present(true);
    flags.set_access_from_all(true);
    let mut chunk = unsafe { Paging256TBChunk::new()? }; // page is not freed

    log!("Allocating page and mapping...");
    let ptr = Box::new("No");
    for i in 0..51200 {
        // 512*512*4
        let address = i * 0x1000;
        if address == 0x1000 {
            chunk.set(address as PageAddress, ptr.as_ptr() as u64 | 0x7, flags)?
        } else {
            chunk.set(address as PageAddress, address, flags)?
        }
    }

    // TODO once chunk.switch() is called, the main kernel page is lost
    log!("Switching to page...");
    unsafe { Paging256TBChunk::switch(chunk) };

    log!("Verifying page mapping...");
    let ptr2 = 0x1000 as *mut char;
    unsafe {
        let c1 = core::ptr::read(ptr2);
        let c2 = core::ptr::read(ptr2.add(1));
        assert!(c1 as char == 'N');
        assert!(c2 as char == 'o');
    }
    log!("Successfully tested paging");
    Ok(())
}
