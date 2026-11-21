thread_local! {
    pub(super) static WORK: std::cell::Cell<[usize; 2]> = const { std::cell::Cell::new([0; 2]) };
}
pub(super) fn record(index: usize) {
    WORK.with(|work| {
        let mut values = work.get();
        values[index] += 1;
        work.set(values);
    });
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "escaped_array_tail_tests.rs"]
mod tests;
