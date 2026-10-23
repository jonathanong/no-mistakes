macro_rules! record_membership_normalization {
    () => {
        tests::record()
    };
}

thread_local! {
    static MEMBERSHIP_NORMALIZATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(super) fn record() {
    MEMBERSHIP_NORMALIZATIONS.with(|count| count.set(count.get() + 1));
}

pub(crate) fn take_membership_normalizations() -> usize {
    MEMBERSHIP_NORMALIZATIONS.with(|count| count.replace(0))
}
