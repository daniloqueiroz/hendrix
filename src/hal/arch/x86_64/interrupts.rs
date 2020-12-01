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
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::kprintln;

use super::gdt::DOUBLE_FAULT_IST_INDEX;
use super::pic_interrupts::{keyboard_interrupt_handler, timer_interrupt_handler, InterruptIndex};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        // configure interruption handlers
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        // interruption handlers for PIC interruptions
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}

/// Load the IDT table to the processor and stores the CPU instance
/// reference to callback on interruptions.
/// This method should be called by the CPU object itself when
/// initializing.
pub fn init_idt() {
    kprintln!("Initializing IDT");
    IDT.load();
}

//# Interruption callbacks - just proxy callbacks to the CPU

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: u64,
) -> ! {
    unsafe {
        // let detail = InterruptionDetails {
        //     error_code: Some(error_code),
        // };
        // CPU_INSTANCE.unwrap().on_interruption(
        //     InterruptionType::DoubleFault,
        //     stack_frame,
        //     Some(detail),
        // );
        // TODO what shall we do here?
        panic!("Kernel panic: Double fault")
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    unsafe {
        // CPU_INSTANCE
        //     .unwrap()
        //     .on_interruption(InterruptionType::Breakpoint, stack_frame, None);
    }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    kprintln!("EXCEPTION: PAGE FAULT");
    kprintln!("Accessed Address: {:?}", Cr2::read());
    kprintln!("Error Code: {:?}", error_code);
    kprintln!("{:#?}", stack_frame);
    panic!("Page fault") // TODO what to do here?
}
