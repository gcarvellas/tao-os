extern crate spin;
extern crate lazy_static;
extern crate volatile;
use crate::{config::{HEAP_ADDRESS, HEAP_BLOCK_SIZE, HEAP_SIZE_BYTES, HEAP_TABLE_ADDRESS}, status::ErrorCode, KERNEL_HEAP};
use super::heap::Heap;
use core::alloc::{Layout, GlobalAlloc};
use core::sync::atomic::{AtomicPtr, Ordering};

pub struct KernelHeap {
    pub heap: Heap
}

impl KernelHeap {
    pub fn default() -> Result<KernelHeap, ErrorCode> {
        let total_table_entries = HEAP_SIZE_BYTES / HEAP_BLOCK_SIZE;
        let end = AtomicPtr::new(unsafe { HEAP_ADDRESS.load(Ordering::Relaxed).add(HEAP_SIZE_BYTES) });
        let _heap = Heap::new(HEAP_ADDRESS, end, total_table_entries, HEAP_TABLE_ADDRESS)?; 
        return Ok(KernelHeap {
            heap: _heap,
        });
    }
}

/**
 * The KernelAllocator references the KERNEL_HEAP because the KERNEL_HEAP static can fail upon
 * initialization. This is possible because KERNEL_HEAP is a lazy_static! but KernelAllocator is
 * not.
 */
struct KernelAllocator;

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        return KERNEL_HEAP.lock().heap.malloc(layout.size()).unwrap();
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        return KERNEL_HEAP.lock().heap.free(ptr);
    }
}

#[global_allocator]
static KERNEL_ALLOCATOR: KernelAllocator = KernelAllocator;
