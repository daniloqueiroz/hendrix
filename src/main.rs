#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(hendrix::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(not(test))]
mod kernel {
    use bootloader::{entry_point, BootInfo};
    use core::panic::PanicInfo;
    use hendrix::arch::x86_64::cpu::CPU;

    use hendrix::kprintln;

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    entry_point!(kernel_main);

    fn kernel_main(_boot_info: &'static BootInfo) -> ! {
        // TODO shall this be moved somewhere? maybe to the kernel module
        kprintln!("Hendrix Kernel {} - Foxy Lady", VERSION);
        let processor = &CPU {};
        processor.init();

        // TODO remove it
        x86_64::instructions::interrupts::int3();

        loop {}
    }

    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        kprintln!("Hendrix is dead: {}", _info);
        loop {}
    }
}

#[cfg(test)]
mod test {
    use bootloader::{entry_point, BootInfo};
    use core::panic::PanicInfo;

    entry_point!(test_kernel_main);

    fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
        hendrix::testing::exit_qemu(hendrix::testing::QemuExitCode::Success);
        loop {}
    }

    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        hendrix::testing::test_panic_handler(info)
    }
}
