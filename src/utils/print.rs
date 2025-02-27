use core::fmt::Write;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::utils::print::_print(format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

pub fn _print(args: core::fmt::Arguments) {
    // Uart is Universal Asynchronous Receiver/Transmitter
    // it is a serial port for communication
    let mut uart = crate::drivers::uart::Uart;

    uart.write_fmt(args).unwrap();
}
