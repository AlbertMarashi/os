//! # Heap Memory Allocator
//!
//! Provides a simple but efficient heap memory allocation system for the kernel.
//! This implementation uses a linked list of free blocks to track available memory.

use core::marker::PhantomData;
use core::mem;
use core::ptr::NonNull;

/// The minimum block size to maintain heap alignment
pub const MIN_BLOCK_SIZE: usize = 16;

/// Block header for our heap allocator
#[repr(C)]
pub struct BlockHeader {
    /// Size of the block including the header
    size: usize,
    /// Pointer to the next free block (only used for free blocks)
    next: Option<NonNull<BlockHeader>>,
}

/// Heap allocator using a simple linked list of free blocks
pub struct Heap {
    /// Head of the free list
    free_list_head: Option<NonNull<BlockHeader>>,
    /// Total size of the heap
    total_size: usize,
    /// Number of allocations (for debugging)
    allocations: usize,
    /// PhantomData to make Heap Send and Sync
    _phantom: PhantomData<BlockHeader>,
}

// Mark our Heap as Send and Sync since we're handling the synchronization
// externally with a Mutex in the GlobalAllocator
unsafe impl Send for Heap {}
unsafe impl Sync for Heap {}

impl Heap {
    /// Creates a new empty heap
    pub const fn new() -> Self {
        Heap {
            free_list_head: None,
            total_size: 0,
            allocations: 0,
            _phantom: PhantomData,
        }
    }

    /// Initializes the heap with the given memory region
    ///
    /// # Safety
    /// This function is unsafe because it requires that the memory region
    /// provided is valid and available for the heap to use exclusively.
    pub unsafe fn init(&mut self, start_addr: usize, size: usize) {
        let aligned_start = align_up(start_addr, mem::align_of::<BlockHeader>());
        let end_addr = start_addr + size;
        let aligned_size = end_addr - aligned_start;

        if aligned_size < mem::size_of::<BlockHeader>() {
            return; // Too small to be useful
        }

        // Create a single free block covering the entire heap
        let block_ptr = aligned_start as *mut BlockHeader;
        *block_ptr = BlockHeader {
            size: aligned_size,
            next: None,
        };

        // Set as the head of our free list
        self.free_list_head = NonNull::new(block_ptr);
        self.total_size = aligned_size;
    }

    /// Allocates memory from the heap
    ///
    /// Returns a pointer to the allocated memory or None if allocation failed
    pub fn allocate(&mut self, layout: core::alloc::Layout) -> Option<NonNull<u8>> {
        // Calculate required size with alignment and header
        let size = layout.size().max(MIN_BLOCK_SIZE);
        let align = layout.align().max(mem::align_of::<BlockHeader>());
        let total_size = size + mem::size_of::<BlockHeader>();

        // Pointer to current block in the free list
        let mut current = self.free_list_head;
        // Pointer to previous block in the free list
        let mut previous: Option<NonNull<BlockHeader>> = None;

        // Search for a suitable block
        while let Some(current_ptr) = current {
            unsafe {
                let current_ref = current_ptr.as_ref();
                if current_ref.size >= total_size {
                    // This block is big enough
                    // Remove it from the free list
                    let next = current_ref.next;
                    if let Some(mut prev) = previous {
                        prev.as_mut().next = next;
                    } else {
                        self.free_list_head = next;
                    }

                    // Calculate aligned address for the allocation
                    let block_start = current_ptr.as_ptr() as usize;
                    let data_start = block_start + mem::size_of::<BlockHeader>();
                    let aligned_data_start = align_up(data_start, align);
                    let adjustment = aligned_data_start - data_start;

                    // If we need to adjust for alignment, we might need to
                    // store a smaller header before the aligned data
                    if adjustment > 0 {
                        // TODO: Implement adjustment for complex alignments
                        // For now, we'll assume the adjustment is small
                    }

                    // If the block is much larger than needed, split it
                    let remaining_size = current_ref.size - total_size;
                    if remaining_size > MIN_BLOCK_SIZE + mem::size_of::<BlockHeader>() {
                        // Split the block
                        let new_block_addr = block_start + total_size;
                        let new_block = new_block_addr as *mut BlockHeader;
                        *new_block = BlockHeader {
                            size: remaining_size,
                            next: self.free_list_head,
                        };
                        self.free_list_head = NonNull::new(new_block);

                        // Update size of the allocated block
                        (*current_ptr.as_ptr()).size = total_size;
                    }

                    // Return pointer to the allocated memory
                    self.allocations += 1;
                    return NonNull::new(
                        (current_ptr.as_ptr() as *mut u8).add(mem::size_of::<BlockHeader>()),
                    );
                }

                // Move to the next block
                previous = Some(current_ptr);
                current = current_ref.next;
            }
        }

        // No suitable block found
        None
    }

    /// Deallocates memory previously allocated by the allocator
    ///
    /// # Safety
    /// This function is unsafe because it requires that the memory was
    /// previously allocated by this allocator and has not been deallocated yet.
    pub unsafe fn deallocate(&mut self, ptr: NonNull<u8>, _layout: core::alloc::Layout) {
        // Get the block header
        let header_addr = (ptr.as_ptr() as usize) - mem::size_of::<BlockHeader>();
        let header_ptr = header_addr as *mut BlockHeader;

        // Add block to the beginning of the free list
        (*header_ptr).next = self.free_list_head;
        self.free_list_head = NonNull::new(header_ptr);
        self.allocations -= 1;

        // TODO: Coalesce adjacent free blocks
    }

    /// Returns the total size of the heap
    pub fn total_size(&self) -> usize {
        self.total_size
    }

    /// Returns the current number of allocations
    pub fn allocation_count(&self) -> usize {
        self.allocations
    }
}

/// Aligns an address upward to the given alignment
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
