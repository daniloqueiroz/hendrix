use lazy_static::lazy_static;

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
