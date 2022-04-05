


// 0x1000_0000 is the base address of the UART
// you can write to the UART with unsafe
pub struct Uart;

impl Uart {

    pub fn write_byte(&self, byte: u8) {
        unsafe {
            (0x1000_0000 as *mut u8).write_volatile(byte);
        }
    }

    pub fn write_string(&self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }
}

impl core::fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
