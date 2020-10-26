#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(hendrix::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(not(test))]
mod kernel {
    use bootloader::{entry_point, BootInfo};
    use core::panic::PanicInfo;
    use hendrix::kprintln;

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    entry_point!(kernel_main);

    fn kernel_main(_boot_info: &'static BootInfo) -> ! {
        kprintln!("Hendrix Kernel {} - Foxy Lady", VERSION);

        loop {}
    }

    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        kprintln!("{}", _info);
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
