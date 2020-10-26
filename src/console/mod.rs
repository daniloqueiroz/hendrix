mod serial;
mod vga;

/// Common implementation of the kprint/kprintln macros.
/// The macros calls the `device_print`, which need to have specific
/// macros to use either serial or vga
#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::console::device_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
    () => ($crate::kprint!("\n"));
    ($fmt:expr) => ($crate::kprint!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::kprint!(
        concat!($fmt, "\n"), $($arg)*));
}

/// device_print implementation using vga mem buffer
#[cfg(not(test))]
#[doc(hidden)]
pub fn device_print(args: ::core::fmt::Arguments) {
    vga::vga_print(args);
}

/// device_print implementation using serial io bus
#[cfg(test)]
#[doc(hidden)]
pub fn device_print(args: ::core::fmt::Arguments) {
    serial::serial_print(args);
}
