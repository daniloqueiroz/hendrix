#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![feature(alloc_error_handler)]
#![test_runner(hendrix::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};

use alloc::prelude::v1::Box;
use hendrix::hal::arch::x86_64::cpu::CPU;
use hendrix::hal::arch::x86_64::memory::Memory;
use hendrix::kernel::{HEAP_SIZE, HEAP_START_ADDRESS};
use hendrix::kprintln;
use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const PROCESSOR: &'static CPU = &CPU {};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // when running in test cfg we can just exit, as the tests are going
    // to be launched from the `lib` module
    #[cfg(test)]
    hendrix::testing::exit_qemu(hendrix::testing::QemuExitCode::Success);

    kprintln!("Hendrix Kernel {} - Foxy Lady", VERSION);

    // Mem and Processor Initializations
    let mut mem = Memory::new(boot_info.physical_memory_offset, &boot_info.memory_map);
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    mem.alloc_frames(VirtAddr::new(HEAP_START_ADDRESS as u64), HEAP_SIZE, flags);

    // TODO to be removed
    mem.print_l4_table();

    PROCESSOR.init();
    let x = Box::new(41);

    // Put the CPU to sleep till we receive the next interruption
    PROCESSOR.hlt_loop()
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprintln!("Hendrix is dead: {}", _info);
    PROCESSOR.hlt_loop()
}
