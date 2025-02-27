use core::fmt::{Display, Write};
use core::sync::atomic::{AtomicUsize, Ordering};

// Track the current section nesting level
static SECTION_LEVEL: AtomicUsize = AtomicUsize::new(0);

// ANSI color codes for terminal output
const COLOR_RESET: &str = "\x1B[0m";

#[derive(Copy, Clone)]
pub enum TextStyle {
    Bold,
    Dim,
    Reset,
}

impl TextStyle {
    fn as_str(&self) -> &'static str {
        match self {
            TextStyle::Bold => "\x1B[1m",
            TextStyle::Dim => "\x1B[2m",
            TextStyle::Reset => COLOR_RESET,
        }
    }
}

impl Display for TextStyle {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Copy, Clone)]
pub enum TextColor {
    Green,
    Blue,
    Yellow,
    Cyan,
    Red,
    Magenta,
    White,
    Reset,
}

impl TextColor {
    fn as_str(&self) -> &'static str {
        match self {
            TextColor::Green => "\x1B[32m",
            TextColor::Blue => "\x1B[34m",
            TextColor::Yellow => "\x1B[33m",
            TextColor::Cyan => "\x1B[36m",
            TextColor::Red => "\x1B[31m",
            TextColor::Magenta => "\x1B[35m",
            TextColor::White => "\x1B[37m",
            TextColor::Reset => COLOR_RESET,
        }
    }
}

impl Display for TextColor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

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

/// Start a section with a header
#[macro_export]
macro_rules! section {
    ($header:expr) => ({
        $crate::utils::print::begin_section($header, format_args!(""));
    });
    ($header:expr, $($arg:tt)*) => ({
        $crate::utils::print::begin_section($header, format_args!($($arg)*));
    });
}

/// End the current section
#[macro_export]
macro_rules! end_section {
    () => {{
        $crate::utils::print::end_section_internal();
    }};
}

/// Print a success message
#[macro_export]
macro_rules! success {
    ($fmt:expr) => ({
        use $crate::utils::print::{TextColor, TextStyle};
        $crate::utils::print::print_colored(format_args!($fmt), TextColor::Green, TextStyle::Bold);
    });
    ($fmt:expr, $($arg:tt)*) => ({
        use $crate::utils::print::{TextColor, TextStyle};
        $crate::utils::print::print_colored(format_args!($fmt, $($arg)*), TextColor::Green, TextStyle::Bold);
    });
}

/// Print an error message
#[macro_export]
macro_rules! error {
    ($fmt:expr) => ({
        use $crate::utils::print::{TextColor, TextStyle};
        $crate::utils::print::print_colored(format_args!($fmt), TextColor::Red, TextStyle::Bold);
    });
    ($fmt:expr, $($arg:tt)*) => ({
        use $crate::utils::print::{TextColor, TextStyle};
        $crate::utils::print::print_colored(format_args!($fmt, $($arg)*), TextColor::Red, TextStyle::Bold);
    });
}

/// Print a warning message
#[macro_export]
macro_rules! warning {
    ($fmt:expr) => ({
        use $crate::utils::print::{TextColor, TextStyle};
        $crate::utils::print::print_colored(format_args!($fmt), TextColor::Yellow, TextStyle::Bold);
    });
    ($fmt:expr, $($arg:tt)*) => ({
        use $crate::utils::print::{TextColor, TextStyle};
        $crate::utils::print::print_colored(format_args!($fmt, $($arg)*), TextColor::Yellow, TextStyle::Bold);
    });
}

/// Print an info message
#[macro_export]
macro_rules! info {
    ($fmt:expr) => ({
        use $crate::utils::print::{TextColor, TextStyle};
        $crate::utils::print::print_colored(format_args!($fmt), TextColor::Cyan, TextStyle::Bold);
    });
    ($fmt:expr, $($arg:tt)*) => ({
        use $crate::utils::print::{TextColor, TextStyle};
        $crate::utils::print::print_colored(format_args!($fmt, $($arg)*), TextColor::Cyan, TextStyle::Bold);
    });
}

/// Print a plain message with no colors
#[macro_export]
macro_rules! msg {
    ($fmt:expr) => ({
        $crate::utils::print::print_plain(format_args!($fmt));
    });
    ($fmt:expr, $($arg:tt)*) => ({
        $crate::utils::print::print_plain(format_args!($fmt, $($arg)*));
    });
}

pub fn _print(args: core::fmt::Arguments) {
    // Uart is Universal Asynchronous Receiver/Transmitter
    // it is a serial port for communication
    let mut uart = crate::drivers::uart::Uart;

    uart.write_fmt(args).unwrap();
}

/// Helper to write the same string multiple times
pub fn write_str_n_times(s: &str, n: usize) {
    for _ in 0..n {
        print!("{}", s);
    }
}

/// Print indentation based on current nesting level (for headers)
pub fn print_indent(level: usize) {
    for i in 0..level {
        if i == level - 1 {
            print!(
                "{}├──{}",
                TextStyle::Dim.as_str(),
                TextStyle::Reset.as_str()
            );
        } else {
            print!(
                "{}│  {}",
                TextStyle::Dim.as_str(),
                TextStyle::Reset.as_str()
            );
        }
    }
}

/// Print indentation for content
pub fn print_content_indent(level: usize) {
    print!("{}", TextStyle::Dim.as_str());
    write_str_n_times("│  ", level);
    print!("{}", TextStyle::Reset.as_str());
}

/// Begin a section with a header and optional message
pub fn begin_section(header: &str, msg: core::fmt::Arguments) {
    let level = SECTION_LEVEL.fetch_add(1, Ordering::SeqCst);

    // Nested section
    print_indent(level);
    print!(
        "{}{}[{}]",
        TextStyle::Bold.as_str(),
        TextColor::Blue.as_str(),
        header
    );

    // Print the message if provided
    let empty = format_args!("");
    if !same_args(&msg, &empty) {
        print!("{}{} ", TextStyle::Dim.as_str(), TextColor::Blue.as_str());
        _print(msg);
    }

    println!("{}", TextStyle::Reset.as_str());
}

/// End a section with no status
pub fn end_section_internal() {
    let _ = SECTION_LEVEL.fetch_sub(1, Ordering::SeqCst);
    // No output for section end
}

/// Function to check if two format_args are the same (empty)
/// This is a hacky way to check if a format_args is empty
fn same_args(a: &core::fmt::Arguments, b: &core::fmt::Arguments) -> bool {
    // Convert both to strings in a temporary buffer
    struct DummyWriter {
        pos: usize,
    }

    impl DummyWriter {
        fn new() -> Self {
            DummyWriter { pos: 0 }
        }
    }

    impl Write for DummyWriter {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            self.pos += s.len();
            Ok(())
        }
    }

    let mut wa = DummyWriter::new();
    let mut wb = DummyWriter::new();

    // Ignore any write errors
    let _ = wa.write_fmt(*a);
    let _ = wb.write_fmt(*b);

    wa.pos == 0 && wb.pos == 0
}

/// Print a message with specified color and style
pub fn print_colored(args: core::fmt::Arguments, color: TextColor, style: TextStyle) {
    let level = SECTION_LEVEL.load(Ordering::SeqCst);

    print_content_indent(level);
    print!("{}{}", color.as_str(), style.as_str());
    _print(args);
    println!("{}", TextStyle::Reset.as_str());
}

/// Print a plain message with no colors
pub fn print_plain(args: core::fmt::Arguments) {
    let level = SECTION_LEVEL.load(Ordering::SeqCst);

    print_content_indent(level);
    _print(args);
    println!("");
}
