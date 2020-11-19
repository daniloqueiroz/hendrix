use x86_64::structures::idt::InterruptStackFrame;

use crate::kprintln;

use super::gdt::init_gdt;
use super::interrupts::init_idt;
use crate::hal::arch::x86_64::pic_interrupts::init_pic;

#[derive(Debug)]
pub enum InterruptionType {
    DoubleFault,
    Breakpoint,
}

pub struct InterruptionDetails {
    pub error_code: Option<u64>,
}

pub struct CPU {}

impl CPU {
    pub fn init(&'static self) {
        init_gdt();
        init_pic();
        init_idt(self);
        x86_64::instructions::interrupts::enable();
    }

    pub(crate) fn on_interruption(
        &self,
        itype: InterruptionType,
        _stack: &mut InterruptStackFrame,
        _details: Option<InterruptionDetails>,
    ) {
        kprintln!("Interruption received: {:?}", itype);
    }

    /// Run an indefinite loop which waits until next interrupt arrives allowing
    /// the CPU to sleep and consume less energy.
    pub fn hlt_loop(&self) -> ! {
        loop {
            x86_64::instructions::hlt();
        }
    }
}
