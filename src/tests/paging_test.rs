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

// TODO this test should revert the addresses and page back to what they originally were
pub fn paging_test() -> Result<(), ErrorCode> {
    log!("Creating a page...");
    let mut flags = PageDirectoryEntry::default();
    flags.set_writeable(true);
    flags.set_present(true);
    flags.set_access_from_all(true);
    let mut chunk = unsafe { Paging256TBChunk::new()? }; // page is not freed

    log!("Allocating page and mapping...");
    // Allocate memory from an area that's one page away
    let ptr = 0x21000 as *mut char;
    unsafe {
        *ptr = 'N';
        *(ptr.offset(1)) = 'o';
    }
    for i in 0..51200 {
        // 512*512*4
        let address = i * 0x1000;
        chunk.set(address as PageAddress, address, flags)?
    }

    chunk.set(0x1000 as PageAddress, 0x21000 as u64, flags)?;

    // TODO once chunk.switch() is called, the main kernel page is lost
    log!("Switching to page...");
    unsafe { Paging256TBChunk::switch(chunk) };

    log!("Verifying page mapping...");
    let ptr2 = 0x1000 as *mut char;
    unsafe {
        *ptr2 = 'A';
        *(ptr2.offset(1)) = 'B';
        let c1 = core::ptr::read(ptr2);
        let c2 = core::ptr::read(ptr2.add(1));

        let c_p1 = core::ptr::read(ptr);
        let c_p2 = core::ptr::read(ptr.add(1));

        assert!(
            c1 == c_p1 && c1 == 'A',
            "Expected mapped value to be 'A' but got '{}'",
            c_p1
        );
        assert!(
            c2 == c_p2 && c2 == 'B',
            "Expected mapped value to be 'B' but got '{}'",
            c_p2
        );
    }
    log!("Successfully tested paging");
    Ok(())
}
