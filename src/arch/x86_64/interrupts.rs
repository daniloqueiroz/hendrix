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
use super::gdt::DOUBLE_FAULT_IST_INDEX;
use crate::arch::x86_64::cpu::{InterruptionDetails, InterruptionType, CPU};
use crate::{kprint, kprintln};
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use pic8259_simple::ChainedPics;
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

/// Starting offset for a primary PIC 8259.
pub const PIC_1_OFFSET: u8 = 32;

/// Starting offset for the secondary PIC 8259.
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// ChainedPics layout with a primary and secondary pic.
///
///                      ____________                          ____________
/// Real Time Clock --> |            |   Timer -------------> |            |
/// ACPI -------------> |            |   Keyboard-----------> |            |      _____
/// Available --------> | Secondary  |----------------------> | Primary    |     |     |
/// Available --------> | Interrupt  |   Serial Port 2 -----> | Interrupt  |---> | CPU |
/// Mouse ------------> | Controller |   Serial Port 1 -----> | Controller |     |_____|
/// Co-Processor -----> |            |   Parallel Port 2/3 -> |            |
/// Primary ATA ------> |            |   Floppy disk -------> |            |
/// Secondary ATA ----> |____________|   Parallel Port 1----> |____________|
///
/// Uses the first free interrupt range from 32 - 47.
pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

/// Enum used for the interrupt offsets.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    /// Timer interrupt uses Line 0 on primary PIC which is offset by PIC_1_OFFSET.
    Timer = PIC_1_OFFSET,

    /// Keyboard interrupt uses Line 1 on primary PIC.
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            // when
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);

        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);

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
    kprintln!("Initializing IDT");
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

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    kprint!(".");

    // Send EOI: end-of-interrupt to the controller indicating the interrupt
    // was processed.
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
        );
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => kprint!("{}", character),
                DecodedKey::RawKey(key) => kprint!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
