#![no_std]
#![no_main]
#![feature(
    panic_info_message,
    asm,
    asm_sym,
    naked_functions
    // llvm_asm
)]

#[naked]
#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    #[rustfmt::skip]
    asm!("
        j {}
        # Disable generation of compressed instructions.
        .option norvc

        # Define a .data section.
        .section .data

        # Define a .text.init section.
        .section .text.init

        # Execution starts here.
        # Any hardware threads (hart) that are not bootstrapping
        # need to wait for an IPI
        csrr	t0, mhartid
        bnez	t0, 3f
        # SATP should be zero, but let's make sure
        csrw	satp, zero
        csrw 	sie, zero

        # Disable linker instruction relaxation for the `la` instruction below.
        # This disallows the assembler from assuming that `gp` is already initialized.
        # This causes the value stored in `gp` to be calculated from `pc`.
        .option push
        .option norelax
            la		gp, _global_pointer
        .option pop
            # Set all bytes in the BSS section to zero.
            la 		a0, _bss_start
            la		a1, _bss_end
            bgeu	a0, a1, 2f
        1:
            sd		zero, (a0)
            addi	a0, a0, 8
            bltu	a0, a1, 1b
        2:
            # Control registers, set the stack, mstatus, mepc,
            # and mtvec to return to the main function.
            # li		t5, 0xffff;
            # csrw	medeleg, t5
            # csrw	mideleg, t5
            la		sp, _stack
            # Setting `mstatus` register:
            # 0b11 << 11: Machine's previous protection mode is 3 (MPP=3).
            # 1 << 7    : Machine's previous interrupt-enable bit is 1 (MPIE=1).
            # 1 << 3    : Machine's interrupt-enable bit is 1 (MIE=1).
            li		t0, (0b11 << 11) | (1 << 7) | (1 << 3)
            csrw	mstatus, t0
            # Machine's exception program counter (MEPC) is set to `kmain`.
            la		t1, {}
            csrw	mepc, t1
            # Machine's trap vector base address is set to `asm_trap_vector`.
            la		t2, 5f
            csrw	mtvec, t2
            # Setting Machine's interrupt-enable bits (`mie` register):
            # 1 << 3 : Machine's M-mode software interrupt-enable bit is 1 (MSIE=1).
            # 1 << 7 : Machine's timer interrupt-enable bit is 1 (MTIE=1).
            # 1 << 11: Machine's external interrupt-enable bit is 1 (MEIE=1).
            li		t3, (1 << 3) | (1 << 7) | (1 << 11)
            csrw	mie, t3
            # Set the return address to infinitely wait for interrupts.
            la		ra, 4f
            # We use mret here so that the mstatus register is properly updated.
            mret
        3:
            # Parked harts go here. We need to set these
            # to only awaken if it receives a software interrupt,
            # which we're going to call the SIPI (Software Intra-Processor Interrupt).
            # We only use these to run user-space programs, although this may
            # change.
        4:
            wfi
            j		4b


        # This will be our trap vector when we start
        # handling interrupts.
        5:
            mret

    ", sym kmain, sym kmain, options(noreturn));
}

// // ///////////////////////////////////
// // / RUST MACROS
// // ///////////////////////////////////
// #[macro_export]
// macro_rules! print
// {
// 	($($args:tt)+) => ({
//         let uart_data = 0x1000_0000 as *mut u8;
//         for c in b"Hello, world!\n" {
//             unsafe { uart_data.write_volatile(*c) };
//         }
// 		// let _ = write!(crate::uart::Uart::new(0x1000_0000), $($args)+);
// 	});
// }
// #[macro_export]
// macro_rules! println
// {
// 	() => ({
// 		print!("\r\n")
// 	});
// 	($fmt:expr) => ({
// 		print!(concat!($fmt, "\r\n"))
// 	});
// 	($fmt:expr, $($args:tt)+) => ({
// 		print!(concat!($fmt, "\r\n"), $($args)+)
// 	});
// }

// #[no_mangle]
// extern "C" fn eh_personality() {}
// #[panic_handler]
// fn panic(info: &core::panic::PanicInfo) -> ! {
//     print!("Aborting: ");
//     if let Some(p) = info.location() {
//         println!(
//             "line {}, file {}: {}",
//             p.line(),
//             p.file(),
//             info.message().unwrap()
//         );
//     } else {
//         println!("no information available.");
//     }
//     abort();
// }
#[no_mangle]
extern "C" fn abort() -> ! {
    loop {
        unsafe {
            asm!("nop");
        }
    }
}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    hcf()
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