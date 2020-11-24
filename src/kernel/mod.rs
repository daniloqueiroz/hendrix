pub mod console;
pub mod heap;

/// Virtual address of the beginning of the Kernel heap
pub const HEAP_START_ADDRESS: usize = 0x_4444_4444_0000;
/// Heap size in Bytes
pub const HEAP_SIZE: usize = 1 * 1024 * 1024; // 1 MiB
/// The buddy allocator allocate memory in blocks
/// This constant defines the block's size
pub const HEAP_LEAF_SIZE: usize = 16;
