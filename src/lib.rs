#![no_std]
#![feature(abi_x86_interrupt)]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(test)]
use core::panic::PanicInfo;

pub mod arch;
pub mod hal;
pub mod kernel;
pub mod testing;

#[cfg(test)]
bootloader::entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    kprintln!("Hendrix Test Runner");
    kprintln!("-------------------");
    test_main();
    testing::exit_qemu(testing::QemuExitCode::Success);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    testing::test_panic_handler(info)
}
