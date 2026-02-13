use x86_64::{
    PhysAddr,
    VirtAddr,
    structures::paging::{
        FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB, PhysFrame, mapper::MapToError
    }
};
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 1000 * 1024; // 1000 KiB

/// A simple frame allocator that iterates over usable memory regions
pub struct BootInfoFrameAllocator {
    memory_regions: &'static [MemoryRegion],
    next_region: usize,
    current_frame: Option<PhysFrame>,
}

/// A memory region descriptor from the bootloader
#[repr(C)]
pub struct MemoryRegion {
    pub start: u64,
    pub end: u64,
    pub kind: u64, // 1 = usable, 0 = reserved
}

impl BootInfoFrameAllocator {
    /// Initialize from a slice of memory regions
    pub unsafe fn init(memory_regions: &'static [MemoryRegion]) -> Self {
        BootInfoFrameAllocator {
            memory_regions,
            next_region: 0,
            current_frame: None,
        }
    }

    /// Get the next usable frame
    fn next_usable_frame(&mut self) -> Option<PhysFrame> {
        loop {
            // If we have a current frame, return it and increment
            if let Some(frame) = self.current_frame {
                let next_start = frame.start_address().as_u64() + 4096;
                let region = &self.memory_regions[self.next_region];

                if next_start < region.end {
                    // still inside this region
                    self.current_frame = Some(PhysFrame::containing_address(PhysAddr::new(next_start)));
                    return Some(frame);
                } else {
                    // move to next region
                    self.next_region += 1;
                    self.current_frame = None;
                }
            } else {
                // find next usable region
                if self.next_region >= self.memory_regions.len() {
                    return None; // no more frames
                }

                let region = &self.memory_regions[self.next_region];
                if region.kind == 1 {
                    // usable
                    self.current_frame = Some(PhysFrame::containing_address(PhysAddr::new(region.start)));
                } else {
                    // skip reserved
                    self.next_region += 1;
                }
            }
        } // loop
    } // fn next_usable_frame
} // impl BootInfoFrameAllocator

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.next_usable_frame()
    }
}

/// Initialize heap using this allocator
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .expect("No more frames!");
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
} // fn init_heap

// helpers for getting stats
pub fn heap_size() -> usize {
    HEAP_SIZE
}

pub fn heap_used() -> usize {
    ALLOCATOR.lock().used()
}

pub fn heap_free() -> usize {
    heap_size() - heap_used()
}

use alloc::string::String;
use alloc::format;
pub fn heap_stat() -> String {
    let used_kib = heap_used() / 1024;
    let total_kib = HEAP_SIZE / 1024;
    format!("Heap: {} / {} KiB", used_kib, total_kib)
}