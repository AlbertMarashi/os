#![no_std]
#![no_main]
// This feature is now stable since Rust 1.81.0
// #![feature(panic_info_message)]
#![feature(allocator_api)]
extern crate alloc;
extern crate lazy_static;

// This is the entry point of the kernel.
//
// Inject the assembly code into the binary.
global_asm!(include_str!("boot.s"));

#[macro_use]
pub mod utils;
mod drivers;
mod kernel;

use core::arch::global_asm;
pub use utils::print;

use crate::print::TextColor;

// Entry point for the kernel
#[no_mangle]
pub extern "C" fn kmain() -> ! {
    utils::welcome::print_welcome_message();

    // Run minimal kernel initialization
    let kernel = kernel::init_kernel();

    section!("KERNEL", "Kernel initialized");
    end_section!();

    // Print counter with both methods
    section!("KERNEL", "LUMINA OS: ENTERING NOOP LOOP");
    end_section!();

    print!("{}", TextColor::Red);

    let mut counter = 0;
    loop {
        // Print a dot directly to UART
        print!(".");

        // Simple delay
        for _ in 0..5_000_000 {
            unsafe { core::arch::asm!("nop") }
        }

        counter += 1;

        // Every 10 iterations, print counter using println
        if counter % 10 == 0 {
            print!("\r\nCount: {}\n", counter);
        }
    }
}

unsafe extern "Rust" {
    pub(crate) unsafe static _stack_start: ();
    pub(crate) unsafe static _stack_end: ();
    pub(crate) unsafe static _memory_start: ();
    pub(crate) unsafe static _memory_end: ();
    pub(crate) unsafe static _bss_start: ();
    pub(crate) unsafe static _bss_end: ();
    pub(crate) unsafe static _heap_start: ();
    pub(crate) unsafe static _heap_size: ();
    pub(crate) unsafe static _data_start: ();
    pub(crate) unsafe static _data_end: ();
}
