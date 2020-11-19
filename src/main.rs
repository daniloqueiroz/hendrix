#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(hendrix::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

use hendrix::hal::arch::x86_64::cpu::CPU;
use hendrix::kprintln;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const PROCESSOR: &'static CPU = &CPU {};

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    // when running in test cfg we can just exit, as the tests are going
    // to be launched from the `lib` module
    #[cfg(test)]
    hendrix::testing::exit_qemu(hendrix::testing::QemuExitCode::Success);

    // TODO shall this be moved somewhere? maybe to the kernel module
    kprintln!("Hendrix Kernel {} - Foxy Lady", VERSION);
    PROCESSOR.init();

    // Put the CPU to sleep till we receive the next interruption
    PROCESSOR.hlt_loop()
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprintln!("Hendrix is dead: {}", _info);
    PROCESSOR.hlt_loop()
}
