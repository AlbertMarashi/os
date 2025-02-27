use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("\n!!! PANIC !!!");
    error!("{}", info);
    error!("!!! PANIC !!!");
    loop {}
}
