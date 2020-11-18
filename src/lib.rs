#![no_std]
#![feature(abi_x86_interrupt)]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod hal;
pub mod kernel;
pub mod testing;

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
use core::panic::PanicInfo;

#[cfg(test)]
#[panic_handler]
/// Panic handler for the tests. Just print the error and exits qemu
/// with an error exit code.
fn panic(info: &PanicInfo) -> ! {
    kprintln!("[failed]\n");
    kprintln!("Error: {}\n", info);
    testing::exit_qemu(testing::QemuExitCode::Failed);
    hlt_loop();
}

/// Run an indefinite lopp which waits until next interrupt arrives allowing
/// the CPU to sleep and consume less energy.
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
