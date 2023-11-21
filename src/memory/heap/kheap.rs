use core::alloc::{Layout, GlobalAlloc};

use crate::{config::{HEAP_SIZE_BYTES, HEAP_BLOCK_SIZE, HEAP_ADDRESS, HEAP_TABLE_ADDRESS}, status::ErrorCode};

use core::sync::atomic::{AtomicPtr, Ordering};
use super::heap::Heap;

extern crate spin;
use spin::Mutex;
extern crate lazy_static;
use lazy_static::lazy_static;
extern crate volatile;

use println;
struct KernelHeap {
    heap: Heap
}

impl KernelHeap {
    fn default() -> Result<KernelHeap, ErrorCode> {
        let total_table_entries = HEAP_SIZE_BYTES / HEAP_BLOCK_SIZE;

        let end = AtomicPtr::new(unsafe { HEAP_ADDRESS.load(Ordering::Relaxed).add(HEAP_SIZE_BYTES) });
        let _heap = Heap::new(HEAP_ADDRESS, end, total_table_entries, HEAP_TABLE_ADDRESS).unwrap(); 
        return Ok(KernelHeap {
            heap: _heap,
        });
    }
}

// The kernel heap init is not a true static since it can fail. The workaround for this is to have
// a separate static KernelAllocator that references the KERNEL_HEAP lazy static
struct KernelAllocator;

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        return KERNEL_HEAP.lock().heap.heap_malloc(layout.size()).unwrap();
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        return KERNEL_HEAP.lock().heap.heap_free(ptr).unwrap();
    }
}

lazy_static! {
    static ref KERNEL_HEAP: Mutex<KernelHeap> = Mutex::new(KernelHeap::default().unwrap());
}

#[global_allocator]
static KERNEL_ALLOCATOR: KernelAllocator = KernelAllocator;


