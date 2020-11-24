//! This module contains Kernel Memory Heap implementation.
//! This module defines Rust's `global_allocator` which is used
//! to allocate/deallocate any heap object.
//! The implementation uses a third party
//! [BuddyAllocator](https://github.com/jjyr/buddy-alloc) library.
//! The allocator takes care only of allocating the virtual memory,
//! and therefore requires that physical memory is already allocated.
use alloc::alloc::{GlobalAlloc, Layout};

use buddy_alloc::buddy_alloc::BuddyAlloc;
use buddy_alloc::BuddyAllocParam;

use crate::commons::Locked;

/// Wraps the 3rd party Allocator implementation.
/// The actual allocator is created in a lazy fashion,
/// So initially it's a `None` Option and only the heap
/// params are defined.
pub struct HeapAllocator {
    buddy_alloc: Option<BuddyAlloc>,
    heap_params: BuddyAllocParam,
}

impl HeapAllocator {
    pub(crate) const fn new(start: usize, size: usize, block_size: usize) -> Self {
        HeapAllocator {
            buddy_alloc: None,
            heap_params: BuddyAllocParam::new(start as *const u8, size, block_size),
        }
    }

    /// This function executes the given closure, which takes the actual
    /// `BuddyAlloc` as parameter.
    /// This mechanism is necessary to make the lazy initialization of the
    /// Allocator.
    fn exec<R, F: FnOnce(&mut BuddyAlloc) -> R>(&mut self, func: F) -> R {
        if self.buddy_alloc.is_none() {
            unsafe {
                // The BuddyAlloc::new is unsafe, cause there's no guarantee
                // that the page frames are actually initialized.
                let alloc = BuddyAlloc::new(self.heap_params);
                self.buddy_alloc = Some(alloc);
            }
        }
        // gets a mutable reference to the allocator
        let mut_alloc = self.buddy_alloc.as_mut().expect("Allocator not present");
        func(mut_alloc)
    }

    /// Returns the number of bytes still available in the heap
    pub fn available_bytes(&mut self) -> usize {
        return self.exec(|alloc| alloc.available_bytes());
    }
    // TODO add a function to retrieve the number of bytes in use
}

/// Implement the `GlobalAlloc` trait for the `Locked<HeapAllocator>`.
/// The `Locked<A>` struct is simply wrapper a type in a `Mutex` to make
/// the `HeapAllocator` thread-safe. This trick is necessary to, once again,
/// be able to initialize the static global allocator.
/// The implementation of both `alloc` and `dealloc` only locks the `HeapAllocator`,
/// and then calls the `exec` method passing a closure that will call the underlying
/// `BuddyAlloc`.
unsafe impl GlobalAlloc for Locked<HeapAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.lock().exec(|alloc| alloc.malloc(layout.size()))
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        self.lock().exec(|alloc| alloc.free(ptr))
    }
}
unsafe impl Sync for Locked<HeapAllocator> {}
