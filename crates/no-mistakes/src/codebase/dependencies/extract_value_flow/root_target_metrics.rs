thread_local! {
    pub(super) static RHS_DISCARDED_WRITES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(super) static CONSTRUCTIONS: std::cell::Cell<[usize; 2]> = const { std::cell::Cell::new([0; 2]) };
}
pub(super) fn record(index: usize) {
    CONSTRUCTIONS.with(|counts| {
        let mut values = counts.get();
        values[index] += 1;
        counts.set(values);
    });
}
