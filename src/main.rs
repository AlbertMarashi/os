#![no_std]
#![no_main]
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

#[no_mangle]
extern "C" fn kmain() -> ! {
    unsafe { (0x1000_0000 as *mut u8).write_volatile(0x42) };
    let uart = crate::drivers::uart::Uart;
    uart.write_string("KERNEL BOOT: Hello from RISC-V!\n");
    println!("LUMINA OS: Kernel Starting...");

    kernel::init_kernel();

    loop {}
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
