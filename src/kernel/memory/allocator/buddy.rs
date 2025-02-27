//! # Buddy Memory Allocator
//!
//! A memory allocator that uses the buddy system algorithm for efficient
//! memory management. This allocator divides memory into blocks of power-of-2 sizes,
//! which allows for efficient splitting and merging of blocks.
//!
//! ## How it works
//!
//! 1. Memory is divided into blocks of sizes 2^0, 2^1, 2^2, ..., 2^MAX_ORDER
//! 2. When memory is requested, find the smallest block that can satisfy the request
//! 3. If no suitable block exists, split a larger block into two "buddies"
//! 4. When memory is freed, check if its buddy is also free
//! 5. If both buddies are free, merge them into a larger block

use core::alloc::Layout;
use core::cmp;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, Ordering};

/// The minimum allocation size (2^MIN_ORDER bytes)
pub const MIN_ORDER: usize = 5; // 32 bytes

/// The maximum allocation size (2^MAX_ORDER bytes)
pub const MAX_ORDER: usize = 15; // 32KB

/// The number of possible block sizes
pub const NUM_ORDERS: usize = MAX_ORDER - MIN_ORDER + 1;

/// The minimum block size (in bytes)
pub const MIN_BLOCK_SIZE: usize = 1 << MIN_ORDER;

/// A free block in the buddy allocator
#[repr(C)]
struct FreeBlock {
    /// Next free block in the free list
    next: Option<NonNull<FreeBlock>>,
}

/// Thread-safe buddy allocator
pub struct BuddyAllocator {
    /// Pointer to the start of the memory region
    memory_start: usize,

    /// Size of the memory region in bytes
    memory_size: usize,

    /// Free lists for each order
    free_lists: [Option<NonNull<FreeBlock>>; NUM_ORDERS],

    /// Whether the allocator has been initialized
    initialized: AtomicBool,
}

unsafe impl Send for BuddyAllocator {}
unsafe impl Sync for BuddyAllocator {}

impl BuddyAllocator {
    /// Creates a new buddy allocator
    pub const fn new() -> Self {
        BuddyAllocator {
            memory_start: 0,
            memory_size: 0,
            free_lists: [None; NUM_ORDERS],
            initialized: AtomicBool::new(false),
        }
    }

    /// Initializes the buddy allocator with a memory region
    ///
    /// # Safety
    ///
    /// This function is unsafe because it requires that the memory region
    /// provided is valid and available for the allocator to use exclusively.
    pub unsafe fn init(&mut self, start_addr: usize, size: usize) {
        // Only initialize once
        if self.initialized.load(Ordering::SeqCst) {
            return;
        }

        // Store the memory region info
        self.memory_start = start_addr;
        self.memory_size = size;

        // Clear the free lists
        for list in &mut self.free_lists {
            *list = None;
        }

        // Initialize the free lists with the available memory
        self.add_memory_region(start_addr, size);

        // Mark as initialized
        self.initialized.store(true, Ordering::SeqCst);
    }

    /// Adds a memory region to the free lists
    ///
    /// This splits the region into power-of-2 sized blocks and adds them to
    /// the appropriate free lists.
    unsafe fn add_memory_region(&mut self, start: usize, size: usize) {
        if size < MIN_BLOCK_SIZE {
            // Too small to be useful
            return;
        }

        // Align the start address
        let aligned_start = align_up(start, MIN_BLOCK_SIZE);

        // Check if alignment moved us past the end of available memory
        if aligned_start >= start + size {
            return;
        }

        // Calculate remaining size after alignment
        let aligned_size = size - (aligned_start - start);

        if aligned_size < MIN_BLOCK_SIZE {
            // Too small after alignment
            return;
        }

        // Keep track of current block
        let mut current_start = aligned_start;
        let mut current_size = aligned_size;

        // Split memory into largest possible power-of-2 blocks
        while current_size >= MIN_BLOCK_SIZE {
            // Find the largest block size that fits
            let order = self.get_order_for_size(current_size);
            // Block size calculation is used to determine actual_block_size below
            let _block_size = 1 << (order + MIN_ORDER);

            // If the block is too big for our max order, use the max order
            let actual_order = cmp::min(order, NUM_ORDERS - 1);
            let actual_block_size = 1 << (actual_order + MIN_ORDER);

            // Add this block to the free list
            self.add_free_block(current_start, actual_order);

            // Move to next portion of memory
            current_start += actual_block_size;

            // Check if we've gone past the end of our memory region
            if current_start >= aligned_start + aligned_size {
                break;
            }

            // Calculate remaining size
            current_size = (aligned_start + aligned_size) - current_start;
        }
    }

    /// Adds a free block to the appropriate free list
    unsafe fn add_free_block(&mut self, addr: usize, order: usize) {
        let block = addr as *mut FreeBlock;

        // Initialize the block
        (*block).next = self.free_lists[order];

        // Add to free list
        self.free_lists[order] = NonNull::new(block);
    }

    /// Gets the order (power of 2) for a given size
    fn get_order_for_size(&self, size: usize) -> usize {
        let size = cmp::max(size, MIN_BLOCK_SIZE);
        let order = (size.next_power_of_two().trailing_zeros() as usize) - MIN_ORDER;
        cmp::min(order, NUM_ORDERS - 1)
    }

    /// Finds the smallest order that can fit the requested size
    fn find_order_for_request(&self, size: usize, align: usize) -> usize {
        // Calculate size required for the allocation
        let required_size = cmp::max(size, align).next_power_of_two();
        let required_size = cmp::max(required_size, MIN_BLOCK_SIZE);

        // Find the appropriate order
        let order = (required_size.trailing_zeros() as usize) - MIN_ORDER;
        cmp::min(order, NUM_ORDERS - 1)
    }

    /// Allocates a block of memory
    ///
    /// Returns a pointer to the allocated memory, or None if allocation failed.
    pub fn allocate(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        if !self.initialized.load(Ordering::SeqCst) {
            return None;
        }

        let required_order = self.find_order_for_request(layout.size(), layout.align());

        // Try to find a free block of the appropriate size
        let mut current_order = required_order;
        while current_order < NUM_ORDERS && self.free_lists[current_order].is_none() {
            current_order += 1;
        }

        if current_order >= NUM_ORDERS {
            // No block large enough
            return None;
        }

        // Get the block from the free list
        let block = unsafe {
            let block = self.free_lists[current_order].unwrap();
            self.free_lists[current_order] = block.as_ref().next;
            block
        };

        // Split the block if it's larger than needed
        unsafe {
            let block_addr = block.as_ptr() as usize;
            let mut block_order = current_order;

            while block_order > required_order {
                // Split the block into two buddies
                block_order -= 1;
                let buddy_size = 1 << (block_order + MIN_ORDER);
                let buddy_addr = block_addr + buddy_size;

                // Add the buddy to the free list
                self.add_free_block(buddy_addr, block_order);
            }

            // Return the allocated block
            NonNull::new(block_addr as *mut u8)
        }
    }

    /// Deallocates a block of memory
    ///
    /// # Safety
    ///
    /// This function is unsafe because it requires that the memory was
    /// previously allocated by this allocator and has not been deallocated yet.
    pub unsafe fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) {
        if !self.initialized.load(Ordering::SeqCst) {
            return;
        }

        let addr = ptr.as_ptr() as usize;
        if addr < self.memory_start || addr >= self.memory_start + self.memory_size {
            // Not in our memory range
            return;
        }

        // Find the order for this allocation
        let mut order = self.find_order_for_request(layout.size(), layout.align());
        let mut block_addr = addr;

        // Try to merge with buddies
        while order < NUM_ORDERS - 1 {
            // Calculate buddy address
            let buddy_bit = 1 << (order + MIN_ORDER);
            let buddy_addr = block_addr ^ buddy_bit;

            // Check if buddy is free
            if !self.is_buddy_free(buddy_addr, order) {
                // Buddy not free, can't merge
                break;
            }

            // Remove buddy from free list
            self.remove_free_block(buddy_addr, order);

            // Merge with buddy (always use the lower address)
            block_addr = cmp::min(block_addr, buddy_addr);
            order += 1;
        }

        // Add the merged block to the free list
        self.add_free_block(block_addr, order);
    }

    /// Checks if a buddy block is free
    unsafe fn is_buddy_free(&self, addr: usize, order: usize) -> bool {
        if addr < self.memory_start || addr >= self.memory_start + self.memory_size {
            return false;
        }

        let mut current = self.free_lists[order];
        while let Some(block) = current {
            if block.as_ptr() as usize == addr {
                return true;
            }
            current = block.as_ref().next;
        }
        false
    }

    /// Removes a block from the free list
    unsafe fn remove_free_block(&mut self, addr: usize, order: usize) {
        let block_ptr = addr as *mut FreeBlock;

        // Handle if it's the head of the list
        if let Some(head) = self.free_lists[order] {
            if head.as_ptr() as usize == addr {
                self.free_lists[order] = head.as_ref().next;
                return;
            }
        }

        // Otherwise, find it in the list
        let mut current = self.free_lists[order];
        while let Some(mut block) = current {
            if let Some(next) = block.as_ref().next {
                if next.as_ptr() as usize == addr {
                    // Found it, remove from list
                    block.as_mut().next = (*block_ptr).next;
                    return;
                }
            }
            current = block.as_ref().next;
        }
    }

    // /// Returns the total size of the memory region
    // pub fn total_size(&self) -> usize {
    //     self.memory_size
    // }

    // /// Returns a heuristic count of current allocations
    // /// by checking how many blocks are NOT in free lists
    // pub fn allocation_count(&self) -> usize {
    //     let mut free_blocks = 0;
    //     for order in 0..NUM_ORDERS {
    //         let mut current = self.free_lists[order];
    //         while let Some(block) = current {
    //             free_blocks += 1;
    //             unsafe {
    //                 current = block.as_ref().next;
    //             }
    //         }
    //     }

    //     // Rough estimate based on memory size and free blocks count
    //     let total_possible_min_blocks = self.memory_size / MIN_BLOCK_SIZE;
    //     let estimated_allocations = total_possible_min_blocks.saturating_sub(free_blocks);

    //     estimated_allocations
    // }
}

/// Aligns an address upward to the given alignment
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
