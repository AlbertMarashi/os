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

pub use utils::print;

use core::arch::global_asm;

// Entry point for the kernel
#[no_mangle]
pub extern "C" fn kmain() -> ! {
    // Test println macro
    println!("===============================");
    println!("LUMINA OS: WELCOME TO LUMINA OS");
    println!("===============================");

    // Run minimal kernel initialization
    kernel::init_kernel();

    // Print counter with both methods
    println!("LUMINA OS: ENTERING NOOP LOOP");

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
            println!("\r\nCount: {}", counter);
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
