//! Hardware Abstraction Layer (hal)
//! This module contains all the low level and architecture specific code.
use lazy_static::lazy_static;

pub mod arch;
mod serial;
mod vga;

#[cfg(not(test))]
lazy_static! {
    pub static ref CONSOLE_IO: vga::VGA = vga::VGA::new();
}

#[cfg(test)]
lazy_static! {
    pub static ref CONSOLE_IO: serial::SerialIO = serial::SerialIO::new();
}
