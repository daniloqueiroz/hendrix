#![no_std]
#![feature(abi_x86_interrupt)]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(alloc_error_handler)]
#![feature(wake_trait)]
#![feature(once_cell)]
#![test_runner(runtime::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

#[cfg(test)]
use {
    crate::runtime::testing,
    bootloader::{entry_point, BootInfo},
    core::panic::PanicInfo,
};

pub mod commons;
pub mod hal;
pub mod kernel;
pub mod runtime;

// Testing entrypoint and panic implementation
#[cfg(test)]
entry_point!(kernel_test_main);

/// Main function when running the tests
/// This will call the test_main which in its turn will
/// call the `testing::test_runner` with all the tests.
#[cfg(test)]
fn kernel_test_main(boot_info: &'static BootInfo) -> ! {
    use {
        crate::hal::arch::x86_64::memory::Memory,
        crate::kernel::{HEAP_SIZE, HEAP_START_ADDRESS},
        x86_64::structures::paging::PageTableFlags,
        x86_64::VirtAddr,
    };

    // To run the test it's required to have memory setup
    let mut mem = Memory::new(boot_info.physical_memory_offset, &boot_info.memory_map);
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    mem.alloc_frames(VirtAddr::new(HEAP_START_ADDRESS as u64), HEAP_SIZE, flags)
        .expect("Unable to allocate virtual memory");

    test_main();

    testing::exit_qemu(testing::QemuExitCode::Success);
    loop {}
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
