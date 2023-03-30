use alloc::alloc::Layout;
use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;

use linked_list_allocator::LockedHeap;
use crate::memory;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; //100 KiB

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation error: {layout:?}")
}

pub fn init_heap() {
    let heap_start = VirtAddr::new(HEAP_START as u64);
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    memory::range_map(heap_start, HEAP_SIZE as u64, Some(flags));
    unsafe {
        ALLOCATOR.lock().init(heap_start.as_mut_ptr(), HEAP_SIZE);
    }
}