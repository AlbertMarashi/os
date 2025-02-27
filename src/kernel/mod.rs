mod memory;
mod process;

use crate::drivers::uart::UART_BASE_ADDR;
use crate::*;
use memory::paging;

pub(crate) fn init_kernel() {
    println!("KERNEL: Initialising kernel...");

    println!(
        "{:#?} {:#?} {:#?} {:#?} {:#?} {:#?} {:#?} {:#?} {:#?} {:#?}",
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

    // Initialise the memory paging system
    let mut page_system = paging::PageSystem::new();

    // Setup a root page table
    page_system.init();

    // Identity map the kernel memory space to the kernel
    let kernal_start = unsafe { &_memory_start as *const _ as u64 };
    let kernal_end = unsafe { &_memory_end as *const _ as u64 };
    match page_system.map_range(
        kernal_start,
        kernal_end,
        kernal_start,
        paging::flags::READ_WRITE_EXECUTE,
    ) {
        Ok(_) => println!("KERNEL: Identity mapped"),
        Err(e) => panic!("KERNEL: Failed to map memory: {}", e),
    }

    // Map the UART for I/O
    match page_system.map_range(
        UART_BASE_ADDR,
        UART_BASE_ADDR + 4096,
        UART_BASE_ADDR,
        paging::flags::READ_WRITE,
    ) {
        Ok(_) => println!("KERNEL: UART mapped"),
        Err(e) => panic!("KERNEL: Failed to map UART: {}", e),
    }

    // Activate the page table
    unsafe {
        page_system.activate();
    }

    println!("KERNEL: Kernel initialised");
}
