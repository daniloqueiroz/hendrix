use crate::hal::arch::x86_64::pic_interrupts::init_pic;
use crate::kernel::cpu::CPU;

use super::gdt::init_gdt;
use super::interrupts::init_idt;

#[derive(Debug)]
pub enum InterruptionType {
    DoubleFault,
    Breakpoint,
}

pub struct InterruptionDetails {
    pub error_code: Option<u64>,
}

pub struct X86CPU {}

impl X86CPU {
    pub fn new() -> Self {
        Self {}
    }
}

impl CPU for X86CPU {
    fn init(&self) {
        init_gdt();
        init_pic();
        init_idt();
        x86_64::instructions::interrupts::enable();
    }

    fn hlt(&self) {
        x86_64::instructions::hlt();
    }
}
