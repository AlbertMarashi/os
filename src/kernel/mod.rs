mod memory;
mod process;

use crate::drivers::uart::UART_BASE_ADDR;
use crate::*;
use memory::paging;
use memory::paging::PAGE_SIZE;

pub(crate) fn init_kernel() {
    println!("KERNEL: Initialising kernel...");

    println!(
        "
_stack_start: {:#?}
_bss_start: {:#?}
_bss_end: {:#?}
_data_start: {:#?}
_memory_start: {:#?}
_stack_end: {:#?}
_heap_start: {:#?}
_heap_size: {:#?}
_data_end: {:#?}
_memory_end: {:#?}",
        core::ptr::addr_of!(_stack_start),
        core::ptr::addr_of!(_bss_start),
        core::ptr::addr_of!(_bss_end),
        core::ptr::addr_of!(_data_start),
        core::ptr::addr_of!(_memory_start),
        core::ptr::addr_of!(_stack_end),
        core::ptr::addr_of!(_heap_start),
        core::ptr::addr_of!(_heap_size),
        core::ptr::addr_of!(_data_end),
        core::ptr::addr_of!(_memory_end)
    );
    // PHASE 1: Create page system
    println!("KERNEL: Creating page system...");
    let mut page_system = paging::PageSystem::new();

    println!("KERNEL: Initialising page system...");

    page_system.init();

    println!("KERNEL: Mapping essential pages individually...");

    // Map code page (where kernel code resides)
    let kernel_start = unsafe { &_memory_start as *const _ as u64 };
    match page_system.map_page(
        kernel_start,
        kernel_start,
        paging::flags::READ_WRITE_EXECUTE,
    ) {
        Ok(_) => println!("  Code page mapped successfully"),
        Err(e) => println!("  Failed to map code page: {}", e),
    }

    // Map stack page (where local variables are stored)
    let stack_start = unsafe { &_stack_start as *const _ as u64 };
    let stack_start_aligned = (stack_start + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
    match page_system.map_page(
        stack_start_aligned,
        stack_start_aligned,
        paging::flags::READ_WRITE,
    ) {
        Ok(_) => println!("  Stack page mapped successfully"),
        Err(e) => println!("  Failed to map stack page: {}", e),
    }

    // Map UART (critical for output)
    match page_system.map_page(UART_BASE_ADDR, UART_BASE_ADDR, paging::flags::READ_WRITE) {
        Ok(_) => println!("  UART mapped successfully"),
        Err(e) => println!("  Failed to map UART: {}", e),
    }

    // PHASE 3: Activate paging with our essential mappings
    println!("KERNEL: Activating paging...");
    unsafe {
        page_system.activate();
    }
    println!("  Paging activated successfully!");

    println!("KERNEL: Initialization complete!");

    // Keep the kernel alive with a simple infinite loop
    loop {
        // Visual indicator that we're still running
        print!(".");
        for _ in 0..10000000 {
            core::hint::spin_loop();
        }
    }
}
