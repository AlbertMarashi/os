#![no_std]
#![no_main]
#![feature(
    panic_info_message,
    asm,
    // llvm_asm,
    // global_asm
)]

// ///////////////////////////////////
// / RUST MACROS
// ///////////////////////////////////
#[macro_export]
macro_rules! print
{
	($($args:tt)+) => ({
        let uart_data = 0x1000_0000 as *mut u8;
        for c in b"Hello, world!\n" {
            unsafe { uart_data.write_volatile(*c) };
        }
		// let _ = write!(crate::uart::Uart::new(0x1000_0000), $($args)+);
	});
}
#[macro_export]
macro_rules! println
{
	() => ({
		print!("\r\n")
	});
	($fmt:expr) => ({
		print!(concat!($fmt, "\r\n"))
	});
	($fmt:expr, $($args:tt)+) => ({
		print!(concat!($fmt, "\r\n"), $($args)+)
	});
}

// ///////////////////////////////////
// / LANGUAGE STRUCTURES / FUNCTIONS
// ///////////////////////////////////
#[no_mangle]
extern "C" fn eh_personality() {}
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("Aborting: ");
    if let Some(p) = info.location() {
        println!(
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        );
    } else {
        println!("no information available.");
    }
    abort();
}
#[no_mangle]
extern "C" fn abort() -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

#[no_mangle]
extern "C" fn kmain() -> ! {
    let uart_data = 0x1000_0000 as *mut u8;
    for c in b"Hello, world!\n" {
        unsafe { uart_data.write_volatile(*c) };
    }

    hcf();
}

fn hcf() -> ! {
    loop {
        unsafe { asm!("nop") };
    }
}

// RUST MODULES

// pub mod uart;
