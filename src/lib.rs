#![no_std]
#![feature(abi_x86_interrupt)]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(alloc_error_handler)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

#[cfg(test)]
use core::panic::PanicInfo;

pub mod commons;
pub mod hal;
pub mod kernel;
pub mod runtime;
pub mod testing;

// TODO can any of this be moved to the runtime module?
// Testing entrypoint and panic implementation
#[cfg(test)]
#[export_name = "_start"]
/// Main function when running the tests
/// This will call the test_main which in its turn will
/// call the `testing::test_runner` with all the tests.
pub extern "C" fn __impl_start() {
    test_main();
    testing::exit_qemu(testing::QemuExitCode::Success);
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
/// Panic handler for the tests. Just print the error and exits qemu
/// with an error exit code.
fn panic(info: &PanicInfo) -> ! {
    kprintln!("[failed]\n");
    kprintln!("Error: {}\n", info);
    testing::exit_qemu(testing::QemuExitCode::Failed);
    loop {}
}
