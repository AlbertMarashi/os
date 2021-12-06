#![no_std]
#![no_main]
#![feature(
    panic_info_message,
    asm,
    global_asm
)]

global_asm!(include_str!("boot.s"));

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let uart = 0x1000_0000 as *mut u8;

    for c in "Kernel Panic!\r\n".chars() {
        unsafe {
            uart.write_volatile(c as u8);
        }
    }
    loop {}
}

#[no_mangle]
extern "C" fn kmain() -> ! {
    // Uart is Universal Asynchronous Receiver/Transmitter
    // 0x1000_0000 is the base address of the UART
    // you can write to the UART with unsafe
    let uart = 0x1000_0000 as *mut u8;

    for c in "Hello, world!\r\n".chars() {
        unsafe {
            uart.write_volatile(c as u8);
        }
    }


    loop {}
}