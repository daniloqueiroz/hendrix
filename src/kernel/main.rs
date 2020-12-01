use bootloader::bootinfo::MemoryMap;
use futures_util::stream::StreamExt;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;

use crate::hal::arch::x86_64::cpu::X86CPU;
use crate::hal::arch::x86_64::memory::Memory;
use crate::kernel::cpu::{CPUEvents, CPU};
use crate::kernel::{HEAP_SIZE, HEAP_START_ADDRESS};
use crate::kprint;
use crate::runtime::executor::EventLoopExecutor;

pub fn kernel_main(memory_offset: u64, mem_map: &'static MemoryMap) -> ! {
    let mut mem = Memory::new(memory_offset, mem_map);
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    mem.alloc_frames(VirtAddr::new(HEAP_START_ADDRESS as u64), HEAP_SIZE, flags)
        .expect("Unable to allocate frames for Kernel heap");

    // TODO to be removed
    mem.print_l4_table();

    let processor = X86CPU::new();
    let mut event_loop = EventLoopExecutor::new();

    // keyboard handler
    let mut keyboard_stream = processor.get_keyboard_stream();
    event_loop.wrap_future(async move {
        let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);

        while let Some(scancode) = keyboard_stream.next().await {
            if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                if let Some(key) = keyboard.process_keyevent(key_event) {
                    match key {
                        DecodedKey::Unicode(character) => kprint!("{}", character),
                        DecodedKey::RawKey(key) => kprint!("{:?}", key),
                    }
                }
            }
        }
    });

    processor.init();
    event_loop.run(|| processor.hlt());

    // TODO shutdown processor
    loop {}
}
