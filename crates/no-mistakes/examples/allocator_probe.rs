use std::hint::black_box;

fn main() {
    // Force-link the library that owns the global allocator registration.
    black_box(no_mistakes::ci_run);
    let mut allocation = Vec::<u64>::with_capacity(4096);
    allocation.push(2120);
    let pointer = black_box(allocation.as_ptr()).cast();
    // SAFETY: the pointer remains live in `allocation`; the query does not read or transfer it.
    let is_mimalloc_heap = unsafe { libmimalloc_sys::mi_is_in_heap_region(pointer) };
    let expected = cfg!(feature = "mimalloc");
    assert_eq!(
        is_mimalloc_heap, expected,
        "global allocator wiring mismatch"
    );
    println!("{{\"expected_mimalloc\":{expected},\"is_mimalloc_heap\":{is_mimalloc_heap}}}");
}
