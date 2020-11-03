//! The interrupts module uses the x86_64 crate to
//! set the processor Interrupt Descriptor Table (IDT).
//! As that table is loaded directly to the processor,
//! it needs to be static.
//!
//! This module also define the callbacks to be called directly by the processor
//! when any interruption happens.
//! To connect back to the CPU object, a CPU instance needs to be passed to
//! the `init_idt` function will hold a static reference to the given object.
//! The callback functions on their turn will just encapsulate the information
//! provides by the x86_64 crate and call the `on_interrupt` method on the CPU
//! object.
//!
//! The main goal of this approach is to keep all the unsafe/low level/idt specific
//! code isolated in this module and having a more high level abstraction,
//! the CPU object, dealing with it.
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::arch::x86_64::cpu::{InterruptionDetails, InterruptionType, CPU};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        // TODO add handlers for the other interruptions
        idt
    };
}

static mut CPU_INSTANCE: Option<&CPU> = None;

/// Load the IDT table to the processor and stores the CPU instance
/// reference to callback on interruptions.
/// This method should be called by the CPU object itself when
/// initializing.
pub fn init_idt(processor: &'static CPU) {
    unsafe {
        CPU_INSTANCE = Some(processor);
    }
    IDT.load();
}

//# Interruption callbacks - just proxy callbacks to the CPU

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: u64,
) -> ! {
    unsafe {
        if CPU_INSTANCE.is_some() {
            let detail = InterruptionDetails {
                error_code: Some(error_code),
            };
            CPU_INSTANCE.unwrap().on_interruption(
                InterruptionType::DoubleFault,
                stack_frame,
                Some(detail),
            );
            // TODO what shall we do here?
            panic!("Kernel panic: Double fault")
        } else {
            panic!("No InterruptionHandler configured");
        }
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    unsafe {
        if CPU_INSTANCE.is_some() {
            CPU_INSTANCE
                .unwrap()
                .on_interruption(InterruptionType::Breakpoint, stack_frame, None);
        } else {
            panic!("No InterruptionHandler configured")
        }
    }
}
