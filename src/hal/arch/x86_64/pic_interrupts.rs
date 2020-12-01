use pic8259_simple::ChainedPics;
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::InterruptStackFrame;

use crate::kernel::cpu_events::add_scancode;
use crate::kprint;

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
static PICS: Mutex<ChainedPics> =
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
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub fn init_pic() {
    unsafe { PICS.lock().initialize() };
}

macro_rules! end_of_interruption {
    ($arg:expr) => {
        unsafe {
            PICS.lock().notify_end_of_interrupt($arg);
        }
    };
}

pub(crate) extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: &mut InterruptStackFrame,
) {
    kprint!(".");
    end_of_interruption!(InterruptIndex::Timer.as_u8());
}

pub(crate) extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: &mut InterruptStackFrame,
) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    add_scancode(scancode);
    end_of_interruption!(InterruptIndex::Keyboard.as_u8());

    // lazy_static! {
    //     static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
    //         Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
    //     );
    // }
    // let mut keyboard = KEYBOARD.lock();
    // if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
    //     if let Some(key) = keyboard.process_keyevent(key_event) {
    //         match key {
    //             DecodedKey::Unicode(character) => kprint!("{}", character),
    //             DecodedKey::RawKey(key) => kprint!("{:?}", key),
    //         }
    //     }
    // }
}
