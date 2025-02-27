mod memory;
mod process;

use crate::drivers::uart::UART_BASE_ADDR;
use crate::*;
use memory::paging;

/// Initialize the kernel
///
/// This function sets up the basic kernel environment:
/// 1. Creates and initializes the paging system
/// 2. Maps the kernel memory using 2MB pages where possible
/// 3. Maps the UART for I/O operations
/// 4. Activates the paging system
/// 5. Initializes the global memory allocator
pub(crate) fn init_kernel() {
    println!("KERNEL: Initializing kernel...");

    // Display memory layout for debugging
    println!(
        "KERNEL: Memory Layout:
    _memory_start: {:#x}
    _memory_end:   {:#x}
    _stack_start:  {:#x}
    _stack_end:    {:#x}
    _heap_start:   {:#x}
    _heap_size:    {:#x}",
        unsafe { &_memory_start as *const _ as u64 },
        unsafe { &_memory_end as *const _ as u64 },
        unsafe { &_stack_start as *const _ as u64 },
        unsafe { &_stack_end as *const _ as u64 },
        unsafe { &_heap_start as *const _ as u64 },
        unsafe { &_heap_size as *const _ as u64 },
    );

    // Phase 1: Initialize paging system
    println!("KERNEL: Creating page system...");
    let mut page_system = paging::PageSystem::new();
    page_system.init();

    // Phase 2: Map kernel memory
    println!("KERNEL: Mapping kernel memory...");
    let kernel_start = unsafe { &_memory_start as *const _ as u64 };
    let kernel_end = unsafe { &_memory_end as *const _ as u64 };
    page_system
        .map_range_optimized(
            kernel_start,
            kernel_end,
            kernel_start,
            paging::flags::READ_WRITE_EXECUTE,
        )
        .expect("Failed to map kernel memory");

    // Phase 3: Map UART for I/O
    println!("KERNEL: Mapping UART...");
    page_system
        .map_page(UART_BASE_ADDR, UART_BASE_ADDR, paging::flags::READ_WRITE)
        .expect("Failed to map UART");

    // Phase 4: Activate paging
    println!("KERNEL: Activating paging...");
    unsafe {
        page_system.activate();
    }

    // Phase 5: Initialize the global memory allocator
    println!("KERNEL: Initializing global memory allocator...");
    unsafe {
        memory::allocator::init_global_allocator();
    }

    // Test allocator with a simple allocation
    println!("KERNEL: Testing allocator...");
    test_allocator();

    println!("KERNEL: Initialization complete!");
}

/// Test the allocator by performing a simple allocation and deallocation
fn test_allocator() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;

    // Test with Box
    let boxed = Box::new(42);
    println!("KERNEL: Successfully allocated Box with value: {}", *boxed);

    // Test with Vec
    let mut vec = Vec::new();
    for i in 0..10 {
        vec.push(i);
    }
    println!(
        "KERNEL: Successfully allocated Vec with length: {}",
        vec.len()
    );
}
