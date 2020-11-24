//! This module contains the low level x86_64 memory manager using pagination.
//! It assumes the full physical memory is mapped by the boot loader with a offset.

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, MapperAllSizes, OffsetPageTable, Page, PageTableFlags, PhysFrame,
    Size4KiB,
};
use x86_64::{structures::paging::PageTable, PhysAddr, VirtAddr};

use crate::kprintln;

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    fn new(memory_map: &'static MemoryMap) -> Self {
        Self {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

/// X86_64 memory - hendrix maps the entire physical memory as virtual,
/// using an offset outside the actual physical memory for doing so.
/// For reduce the number of tables to map the entire memory,
/// this mapping uses `Huge Pages` (pages larger than 4KiB).
pub struct Memory {
    physical_memory_offset: PhysAddr,
    mapper: OffsetPageTable<'static>,
    frame_allocator: BootInfoFrameAllocator,
}

impl Memory {
    /// Initializing the `Memory` struct informing what is the offset for the
    /// physical memory mapping.
    pub fn new(memory_offset: u64, mem_map: &'static MemoryMap) -> Self {
        let (level_4_table_frame, _) = Cr3::read();
        let virt = VirtAddr::new(memory_offset + level_4_table_frame.start_address().as_u64());
        let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
        unsafe {
            return Self {
                physical_memory_offset: PhysAddr::new(memory_offset),
                mapper: OffsetPageTable::new(&mut *page_table_ptr, VirtAddr::new(memory_offset)),
                frame_allocator: BootInfoFrameAllocator::new(mem_map),
            };
        }
    }

    pub fn alloc_frames(
        &mut self,
        start_address: VirtAddr,
        size: usize,
        flags: PageTableFlags,
    ) -> Result<(), MapToError<Size4KiB>> {
        let page_range = {
            let end_address = start_address + size - 1u64;
            let first_page = Page::containing_address(start_address);
            let last_page = Page::containing_address(end_address);
            Page::range_inclusive(first_page, last_page)
        };

        for page in page_range {
            let frame = self
                .frame_allocator
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;

            unsafe {
                self.mapper
                    .map_to(page, frame, flags, &mut self.frame_allocator)?
                    .flush();
            }
        }

        Ok(())
    }

    // TODO do i need any of the functions below?

    /// Translates a physical address to its virtual address. The translation
    /// is made by simply adding the `physical_memory_offset` to the physical address
    fn translate_physical_to_virtual(&self, physical_address: u64) -> VirtAddr {
        return VirtAddr::new(physical_address + &self.physical_memory_offset.as_u64());
    }

    pub fn print_l4_table(&self) {
        let l4_table = unsafe { self.get_active_level_4_table() };

        for (i, entry) in l4_table.iter().enumerate() {
            if !entry.is_unused() {
                kprintln!("L4 Entry {}: {:?}", i, entry);
                // get the physical address from the entry and convert it
                let phys = entry.frame().unwrap().start_address();
                let virt = self.translate_physical_to_virtual(phys.as_u64());
                let l3_table: &PageTable = unsafe { &*virt.as_mut_ptr() };

                // print non-empty entries of the level 3 table
                for (i, entry) in l3_table.iter().enumerate() {
                    if !entry.is_unused() {
                        kprintln!("  L3 Entry {}: {:?}", i, entry);
                    }
                }
            }
        }

        let addresses = [
            // the identity-mapped vga buffer page
            0xb8000,
            // some code page
            0x201008,
            // some stack page
            0x0100_0020_1a10,
            // virtual address mapped to physical address 0
            self.physical_memory_offset.as_u64(),
        ];
        for &address in &addresses {
            let virt = VirtAddr::new(address);
            let phys = self.mapper.translate_addr(virt);
            kprintln!("{:?} -> {:?}", virt, phys);
        }
    }

    /// Returns a mutable reference to the active level 4 table.
    ///
    /// This function is unsafe because the caller must guarantee that the
    /// complete physical memory is mapped to virtual memory at the passed
    /// `physical_memory_offset`. Also, this function must be only called once
    /// to avoid aliasing `&mut` references (which is undefined behavior).
    unsafe fn get_active_level_4_table(&self) -> &'static mut PageTable {
        let (level_4_table_frame, _) = Cr3::read();

        let phys = level_4_table_frame.start_address();
        let virt = self.translate_physical_to_virtual(phys.as_u64());
        let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

        &mut *page_table_ptr // unsafe
    }
}
