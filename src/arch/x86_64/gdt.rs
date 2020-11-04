//! This module uses the x86_64 crate to define a
//! Task State Segment (TSS) to configure the
//! Interrupt Stack Table - which is used by the
//! processor to switch to a clean stack when handling
//! an interruption.
//! This module then defined a Global Descriptor Table (GDT),
//! which uses the TSS previously create, and then load
//! it to the processor.
//!
//! As with the `interrupt` module, the goal of this module
//! is to encapsulate this low level operations and provide
//! a single interface for the `CPU` to initialize those
//! data structures.
use lazy_static::lazy_static;
use x86_64::instructions::segmentation::set_cs;
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

use crate::kprintln;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

/// TODO describe what is each of those SegmentSelectors about
struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            /// Defines the memory address for the stack to be used when
            /// a double fault interruption happens.
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
            },
        )
    };
}

/// Initialize the Global Descriptor Table by defining a TSS,
/// assigning the interrupt_stack_table to be used when a double fault
/// interrupt happens, configuring the GDT to use the newly created
/// TSS and finally loading the GDT.
pub fn init_gdt() {
    kprintln!("Initializing GDT");
    GDT.0.load();
    unsafe {
        set_cs(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
