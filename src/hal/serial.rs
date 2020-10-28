use core::fmt::{Arguments, Write};

use spin::Mutex;
use uart_16550::SerialPort;

use crate::kernel::console::ConsolePrinter;

pub struct SerialIO {
    serial_writer: Mutex<SerialPort>,
}

impl SerialIO {
    pub fn new() -> Self {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        return Self {
            serial_writer: Mutex::new(serial_port),
        };
    }
}

impl ConsolePrinter for SerialIO {
    fn print(&self, args: Arguments) {
        self.serial_writer
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    }
}
