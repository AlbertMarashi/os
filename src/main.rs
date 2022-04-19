#![no_std]
#![no_main]
#![feature(
    default_alloc_error_handler,
    panic_info_message,
)]

extern crate alloc;
extern crate lazy_static;

#[macro_use] mod utils;
mod drivers;
mod memory;


use core::{arch::global_asm};

global_asm!(include_str!("boot.s"));

#[no_mangle]
extern "C" fn kmain() -> ! {
    println!("Hello World!");
    unsafe { println!("{:#?} {:#?} {:#?} {:#?} {:#?} {:#?} {:#?} {:#?} {:#?} {:#?}",
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
    );}

    // let page_system = memory::page::PageSystem::new();

    // println!("{:#?}", page_system);

    loop {}
}

extern "Rust" {
    static _stack_start: ();
    static _stack_end: ();
    static _memory_start: ();
    static _memory_end: ();
    static _bss_start: ();
    static _bss_end: ();
    static _heap_start: ();
    static _heap_size: ();
    static _data_start: ();
    static _data_end: ();
}