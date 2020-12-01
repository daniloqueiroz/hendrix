#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![feature(alloc_error_handler)]
#![test_runner(hendrix::runtime::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

use hendrix::kernel::main::kernel_main;
use hendrix::kprintln;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

entry_point!(start);

fn start(boot_info: &'static BootInfo) -> ! {
    // when running in test cfg we can just exit, as the tests are going
    // to be launched from the `lib` module
    #[cfg(test)]
    hendrix::runtime::testing::exit_qemu(hendrix::runtime::testing::QemuExitCode::Success);

    kprintln!("Hendrix Kernel {} - Foxy Lady", VERSION);
    kernel_main(boot_info.physical_memory_offset, &boot_info.memory_map)
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprintln!("Kernel Panic: {}", _info);
    loop {}
}
