#![no_std]
#![no_main]
// This feature is now stable since Rust 1.81.0
// #![feature(panic_info_message)]
// extern crate alloc;
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

// Direct write to the UART for early debugging
fn direct_uart_write(c: u8) {
    unsafe {
        // QEMU UART address
        let uart_addr: *mut u8 = 0x1000_0000 as *mut u8;
        // Write directly to UART data register
        core::ptr::write_volatile(uart_addr, c);
    }
}

// Write a string directly to UART
fn direct_print(s: &str) {
    for c in s.bytes() {
        direct_uart_write(c);
    }
}

// Entry point for the kernel
#[no_mangle]
pub extern "C" fn kmain() -> ! {
    // Direct UART debug at start of kmain
    direct_print("[KMAIN-START]\r\n");

    // Test println macro
    println!("LUMINA OS: Basic Kernel Test");

    // Run minimal kernel initialization
    kernel::init_kernel();

    // Print counter with both methods
    println!("LUMINA OS: Testing print loop...");

    let mut counter = 0;
    loop {
        // Print a dot directly to UART
        direct_uart_write(b'.');

        // Simple delay
        for _ in 0..1_000_000 {
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
