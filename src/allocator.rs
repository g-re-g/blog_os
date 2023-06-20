use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

// Arbitrary starting point for the Kernel's heap. 0x4444_4444_0000 chosen
// because it is a) unused, and b) easy to identify
pub const HEAP_START: usize = 0x_4444_4444_0000;
// Kernel's heap size; 100 KiB. If we need more we can always change this.
pub const HEAP_SIZE: usize = 100 * 1024;

// The #[global_allocator] attribute tells rust what static item we want to use
// as our OS's global memory allocator. It must implement the
// alloc::alloc::GlobalAlloc trait.
use linked_list_allocator::LockedHeap;
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Initializing the Kernel's heap. We need to do this, of course, so we can
// store data for the Kernel itself on a heap.
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    // Create a PageRangeInclusive to describe our kernel's heap.
    // https://docs.rs/x86_64/latest/x86_64/structures/paging/page/struct.PageRangeInclusive.html
    let page_range = {
        // The virtual address where our Kernel's heap starts.
        let heap_start = VirtAddr::new(HEAP_START as u64);
        // The Virtual address where it ends, end inclusive
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        // Allign the start virtual address to a page boundary.
        let heap_start_page = Page::<Size4KiB>::containing_address(heap_start);
        // Align the end virtual address to a page boundary.
        let heap_end_page = Page::<Size4KiB>::containing_address(heap_end);
        // Construct an inclusive page range.
        Page::<Size4KiB>::range_inclusive(heap_start_page, heap_end_page)
    };

    // Initialize all the TODO
    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        // Create the page table entry for the page->frame mapping
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    // Initialize our heap allocator to the correct heap location
    // after we have initialized the page table
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

#[test_case]
fn heap_allocation_and_freeing() {
    use alloc::boxed::Box;
    use alloc::format;
    // Start of address space
    let heap = Box::new(41);
    assert_eq!(format!("{:p}", heap), format!("0x{:x}", HEAP_START));
    // Next 16 bytes
    let heap = Box::new(42);
    assert_eq!(format!("{:p}", heap), format!("0x{:x}", HEAP_START + 16));
    // Free up 16 bytes
    drop(heap);
    // Reuse previous 16 bytes
    let heap = Box::new(42);
    assert_eq!(format!("{:p}", heap), format!("0x{:x}", HEAP_START + 16));
}
