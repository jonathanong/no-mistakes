thread_local! {
    pub(in super::super) static ASSIGNMENT_CONTEXT_CHECKS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(in super::super) static SITES: std::cell::Cell<[usize; 6]> = const { std::cell::Cell::new([0; 6]) };
}
pub(in super::super) fn site(index: usize) {
    SITES.with(|counts| {
        let mut values = counts.get();
        values[index] += 1;
        counts.set(values);
    });
}
