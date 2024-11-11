use crate::println;
use crate::KERNEL_HEAP;
use alloc::format;
use core::alloc::GlobalAlloc;
use core::alloc::Layout;

macro_rules! log {
    ($($arg:tt)*) => {
        println!("[malloc_test] {}", format!($($arg)*));
    };
}

pub fn malloc_test() {
    log!("Allocating heap memory...");

    let layout = Layout::new::<i32>();

    let ptr = unsafe { KERNEL_HEAP.alloc(layout) as *mut i32 };
    if ptr.is_null() {
        panic!("Failed to allocate memory");
    }

    log!("Freeing heap memory...");
    unsafe {
        KERNEL_HEAP.dealloc(ptr as *mut u8, layout);
    }

    log!("Allocating again should be at the same address...");
    let ptr2 = unsafe { KERNEL_HEAP.alloc(layout) as *mut i32 };
    assert!(ptr == ptr2);

    log!("Allocating again should be at a different address...");
    let ptr3 = unsafe { KERNEL_HEAP.alloc(layout) as *mut i32 };
    assert!(ptr2 != ptr3);

    unsafe {
        KERNEL_HEAP.dealloc(ptr2 as *mut u8, layout);
        KERNEL_HEAP.dealloc(ptr3 as *mut u8, layout);
    }

    log!("Successfully tested the heap");
}
