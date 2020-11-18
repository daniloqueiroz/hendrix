use x86_64::structures::idt::InterruptStackFrame;

use crate::kprintln;

use super::gdt::init_gdt;
use super::interrupts::{init_idt, init_pic};

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
        init_idt(self);
        init_pic();
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
}
