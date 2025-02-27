use super::{devices, memory::paging};
use crate::{
    _heap_size, _heap_start, _memory_end, _memory_start, _stack_end, _stack_start,
    drivers::uart::UART_BASE_ADDR, error, kernel::memory::allocator, success,
};

#[derive(Debug)]
#[allow(unused)]
pub struct Kernel {
    pub page_system: paging::PageSystem,
    pub device_manager: devices::DeviceManager,
}

/// Initialize the kernel
///
/// This function sets up the basic kernel environment:
/// 1. Creates and initializes the paging system
/// 2. Maps the kernel memory using 2MB pages where possible
/// 3. Maps the UART for I/O operations
/// 4. Maps the VGA buffer for text display
/// 5. Activates the paging system
/// 6. Initializes the global memory allocator
/// 7. Initializes the VGA text mode driver
/// 8. Discovers hardware devices
pub(crate) fn init_kernel() -> Kernel {
    section!("KERNEL", "Initializing Kernel");

    print_memory_layout();

    // Phase 1: Initialize paging system
    let page_system = init_paging_system();

    // Phase 2: Initialise global memory allocation system
    allocator::init_global_allocator();

    // Phase 3: Device tree discovery
    let device_manager = discover_devices();

    success!("Kernel initialization complete");

    end_section!();

    Kernel {
        page_system,
        device_manager,
    }
}

/// Discover hardware devices in the system
fn discover_devices() -> devices::DeviceManager {
    section!(
        "DEVICE TREE",
        "Discovering devices and initializing device manager"
    );
    let mut device_manager = devices::DeviceManager::new();
    device_manager.discover_devices();
    success!("Found {} devices", device_manager.num_devices());
    end_section!();
    device_manager
}

fn print_memory_layout() {
    section!("KERNEL", "Memory Layout");
    msg!("_memory_start: {:#x}", unsafe {
        &_memory_start as *const _ as u64
    });
    msg!("_memory_end: {:#x}", unsafe {
        &_memory_end as *const _ as u64
    });
    msg!("_stack_start: {:#x}", unsafe {
        &_stack_start as *const _ as u64
    });
    msg!("_stack_end: {:#x}", unsafe {
        &_stack_end as *const _ as u64
    });
    msg!("_heap_start: {:#x}", unsafe {
        &_heap_start as *const _ as u64
    });
    msg!("_heap_size: {:#x}", unsafe {
        &_heap_size as *const _ as u64
    });
    end_section!();
}

fn init_paging_system() -> paging::PageSystem {
    section!("PAGING", "Initializing Paging System");
    let mut page_system = paging::PageSystem::new();
    page_system.init();

    // Map kernel memory
    map_kernel_memory(&mut page_system);

    // Map UART for I/O
    map_uart(&mut page_system);

    // Activate paging
    section!("PAGING", "Activating Paging");
    unsafe {
        page_system.activate();
    }
    success!("Paging activated");
    end_section!();
    end_section!();
    page_system
}

fn map_kernel_memory(page_system: &mut paging::PageSystem) {
    section!("MEMORY MAPPING", "Kernel Memory");
    let kernel_start = unsafe { &_memory_start as *const _ as u64 };
    let kernel_end = unsafe { &_memory_end as *const _ as u64 };

    match page_system.map_range_optimized(
        kernel_start,
        kernel_end,
        kernel_start,
        paging::flags::READ_WRITE_EXECUTE,
    ) {
        Ok(_) => success!(
            "Kernel memory mapped: {:#x} - {:#x}",
            kernel_start,
            kernel_end
        ),
        Err(_) => error!("Failed to map kernel memory"),
    }

    end_section!();
}

fn map_uart(page_system: &mut paging::PageSystem) {
    section!("MEMORY MAPPING", "UART");
    match page_system.map_page(UART_BASE_ADDR, UART_BASE_ADDR, paging::flags::READ_WRITE) {
        Ok(_) => success!("UART mapped at {:#x}", UART_BASE_ADDR),
        Err(_) => error!("Failed to map UART"),
    }
    end_section!();
}
