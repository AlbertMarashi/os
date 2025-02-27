use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\n!!! PANIC !!!");
    println!("{}", info);

    loop {}
}
