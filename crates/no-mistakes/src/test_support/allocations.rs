//! Count actual allocation requests only on the observing unit-test thread.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    static COUNT: Cell<Option<usize>> = const { Cell::new(None) };
}

struct ObservedSystem;

#[global_allocator]
static ALLOCATOR: ObservedSystem = ObservedSystem;

fn allocated(pointer: *mut u8) {
    if !pointer.is_null() {
        let _ = COUNT.try_with(|count| {
            if let Some(value) = count.get() {
                count.set(Some(value + 1));
            }
        });
    }
}

// SAFETY: every operation forwards the original pointer/layout to System.
unsafe impl GlobalAlloc for ObservedSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        allocated(pointer);
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        allocated(pointer);
        pointer
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let pointer = unsafe { System.realloc(pointer, layout, size) };
        allocated(pointer);
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
    }
}

pub(crate) fn observe<T>(run: impl FnOnce() -> T) -> (T, usize) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            COUNT.set(None);
        }
    }
    assert!(
        COUNT.replace(Some(0)).is_none(),
        "nested allocation observation"
    );
    let reset = Reset;
    let result = run();
    let count = COUNT.take().unwrap();
    drop(reset);
    (result, count)
}
