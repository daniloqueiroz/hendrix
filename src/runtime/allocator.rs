// Rust runtime implementation
use crate::commons::Locked;
use crate::kernel::heap::HeapAllocator;
use crate::kernel::{HEAP_LEAF_SIZE, HEAP_SIZE, HEAP_START_ADDRESS};

// Rust allocator configuration
#[global_allocator]
static ALLOCATOR: Locked<HeapAllocator> = Locked::new(HeapAllocator::new(
    HEAP_START_ADDRESS,
    HEAP_SIZE,
    HEAP_LEAF_SIZE,
));

// Error handler for memory allocation errors
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
