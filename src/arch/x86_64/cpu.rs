use super::interrupts::init_idt;
use crate::kprintln;
use x86_64::structures::idt::InterruptStackFrame;

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
        init_idt(self);
    }

    pub(crate) fn on_interruption(
        &self,
        itype: InterruptionType,
        stack: &mut InterruptStackFrame,
        details: Option<InterruptionDetails>,
    ) {
        kprintln!("Interruption received: {:?}", itype);
    }
}
