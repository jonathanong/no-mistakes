thread_local! {
    pub(in super::super) static OWNED_KEYS: std::cell::Cell<[usize; 4]> = const { std::cell::Cell::new([0; 4]) };
    pub(in super::super) static CHILD_KEYS: std::cell::Cell<[usize; 4]> = const { std::cell::Cell::new([0; 4]) };
    pub(in super::super) static PATH_DESCENTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(in super::super) fn child_lookup(occupied: bool) {
    CHILD_KEYS.with(|counts| {
        let mut values = counts.get();
        values[usize::from(!occupied)] += 1;
        counts.set(values);
    });
}

pub(in super::super) fn child_copy(member: &str) {
    CHILD_KEYS.with(|counts| {
        let mut values = counts.get();
        values[2] += 1;
        values[3] += member.len();
        counts.set(values);
    });
}

pub(in super::super) fn owned_key(index: usize, name: &str) {
    OWNED_KEYS.with(|counts| {
        let mut values = counts.get();
        values[index] += 1;
        values[index + 1] += name.len();
        counts.set(values);
    });
}
