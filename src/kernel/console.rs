pub trait ConsolePrinter {
    /// print a fmt string to Console
    fn print(&self, args: ::core::fmt::Arguments);
}

/// Common implementation of the kprint/kprintln macros.
/// The macros calls the `device_print`, which need to have specific
/// macros to use either serial or vga
#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::kernel::console::device_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
    () => ($crate::kprint!("\n"));
    ($fmt:expr) => ($crate::kprint!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::kprint!(
        concat!($fmt, "\n"), $($arg)*));
}

pub fn device_print(args: ::core::fmt::Arguments) {
    use crate::hal::CONSOLE_IO;
    CONSOLE_IO.print(args);
}
