

// task data structures:

use core::alloc::GlobalAlloc;

use alloc::vec::Vec;

struct Allocator {
    tasks: Vec<Task>,
}

struct Task {
    root_table: u64,
}


#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator::new();

struct SimpleAllocator;

impl SimpleAllocator {
    const fn new() -> Self {
        Self
    }
}

unsafe impl GlobalAlloc for SimpleAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        todo!()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        todo!()
    }
}