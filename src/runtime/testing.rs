//! This module contains the testing framework support
//! to be able to write and run tests for Hendrix.
use crate::{kprint, kprintln};

pub trait Testable {
    fn run(&self) -> ();
}

/// Decorator pattern to print the name of the
/// test function and its status
impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        kprint!("{}...", core::any::type_name::<T>());
        self();
        kprintln!("[ok]");
    }
}

/// The test runner iterates over all received tests and
/// executes them.
pub fn test_runner(tests: &[&dyn Testable]) {
    kprintln!("Hendrix Test Runner");
    kprintln!("-------------------");
    kprintln!(">> Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}
