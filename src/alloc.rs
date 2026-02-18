use buddy_alloc::{BuddyAllocParam, FastAllocParam, NonThreadsafeAlloc};
use core::ptr::addr_of;

// These values can be tuned
const FAST_HEAP_SIZE: usize = 4 * 1024; // 4 KB
const HEAP_SIZE: usize = 16 * 1024; // 16 KB
const LEAF_SIZE: usize = 16;

static mut FAST_HEAP: [u8; FAST_HEAP_SIZE] = [0u8; FAST_HEAP_SIZE];
static mut HEAP: [u8; HEAP_SIZE] = [0u8; HEAP_SIZE];

#[global_allocator]
static ALLOC: NonThreadsafeAlloc = {
    // let fast_param = FastAllocParam::new(FAST_HEAP.as_ptr(), FAST_HEAP_SIZE);
    // let buddy_param = BuddyAllocParam::new(HEAP.as_ptr(), HEAP_SIZE, LEAF_SIZE);
    let fast_param = FastAllocParam::new(addr_of!(FAST_HEAP) as *const u8, FAST_HEAP_SIZE);
    let buddy_param = BuddyAllocParam::new(addr_of!(HEAP) as *const u8, HEAP_SIZE, LEAF_SIZE);
    NonThreadsafeAlloc::new(fast_param, buddy_param)
};
