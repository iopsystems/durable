use std::alloc::System;

struct DurableAllocator(System);

unsafe impl std::alloc::GlobalAlloc for DurableAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        self.0.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        self.0.dealloc(ptr, layout)
    }

    unsafe fn alloc_zeroed(&self, layout: std::alloc::Layout) -> *mut u8 {
        self.0.alloc_zeroed(layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: std::alloc::Layout, new_size: usize) -> *mut u8 {
        self.0.realloc(ptr, layout, new_size)
    }
}

#[global_allocator]
static ALLOCATOR: DurableAllocator = DurableAllocator(System);
