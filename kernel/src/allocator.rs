use core::{
    alloc::{GlobalAlloc, Layout},
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::__heap_base;

pub struct KernelAlloc;

static HEAP_PTR: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for KernelAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();

        let mut ptr = HEAP_PTR.load(Ordering::Relaxed);
        if ptr == 0 {
            unsafe {
                ptr = &__heap_base as *const u8 as usize;
            }
        }

        let aligned = (ptr + align - 1) & !(align - 1);
        let next = aligned + size;

        HEAP_PTR.store(next, Ordering::Relaxed);
        aligned as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}
