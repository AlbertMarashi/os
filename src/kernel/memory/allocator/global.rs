//! # Global Allocator
//!
//! This module implements the global allocator for the kernel,
//! which provides memory allocation services for the Rust standard library.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, Ordering};

use super::buddy::BuddyAllocator;

// Minimum heap size to start with (can be expanded later)
// pub const HEAP_SIZE: usize = 1024 * 1024; // 1MB

/// Global allocator for the kernel
pub struct GlobalAllocator {
    /// The actual heap implementation
    allocator: spin::Mutex<BuddyAllocator>,
    /// Flag to indicate if the heap has been initialized
    initialized: AtomicBool,
}

// Global allocator instance
#[global_allocator]
pub static ALLOCATOR: GlobalAllocator = GlobalAllocator::new();

impl GlobalAllocator {
    /// Creates a new GlobalAllocator
    const fn new() -> Self {
        GlobalAllocator {
            allocator: spin::Mutex::new(BuddyAllocator::new()),
            initialized: AtomicBool::new(false),
        }
    }

    /// Initializes the heap if not already initialized
    ///
    /// # Safety
    /// This function is unsafe because it requires that the memory address is valid
    /// and available for the heap to use exclusively.
    pub unsafe fn init(&self, heap_start: usize, heap_size: usize) {
        // Only initialize once
        if self.initialized.load(Ordering::SeqCst) {
            return;
        }

        // Initialize the heap
        self.allocator.lock().init(heap_start, heap_size);

        // Mark as initialized
        self.initialized.store(true, Ordering::SeqCst);
    }
}

/// Initialize the global allocator
///
/// # Safety
/// This function is unsafe because it requires that the memory region
/// provided is valid and available for the heap to use exclusively.
pub unsafe fn init_global_allocator() {
    unsafe extern "Rust" {
        unsafe static _heap_start: usize;
        unsafe static _heap_size: usize;
    }

    let heap_start = &_heap_start as *const _ as usize;
    let heap_size = &_heap_size as *const _ as usize;

    ALLOCATOR.init(heap_start, heap_size);
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Check if heap is initialized
        if !self.initialized.load(Ordering::SeqCst) {
            return core::ptr::null_mut();
        }

        // Allocate memory from the heap
        self.allocator
            .lock()
            .allocate(layout)
            .map_or(core::ptr::null_mut(), |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Check if heap is initialized and ptr is not null
        if !self.initialized.load(Ordering::SeqCst) || ptr.is_null() {
            return;
        }

        // Deallocate memory
        if let Some(non_null_ptr) = NonNull::new(ptr) {
            self.allocator.lock().deallocate(non_null_ptr, layout);
        }
    }
}
