use std::cell::Cell;

thread_local! {
    static SCHEMAS: Cell<usize> = const { Cell::new(0) };
    static DEEP_SCHEMAS: Cell<usize> = const { Cell::new(0) };
    static COMPARISONS: Cell<usize> = const { Cell::new(0) };
}

pub(super) fn schema_visit(depth: usize) {
    SCHEMAS.with(|count| count.set(count.get() + 1));
    if depth > 1 {
        DEEP_SCHEMAS.with(|count| count.set(count.get() + 1));
    }
}

pub(super) fn comparison() {
    COMPARISONS.with(|count| count.set(count.get() + 1));
}

pub(super) fn counts() -> (usize, usize, usize) {
    (
        SCHEMAS.with(Cell::get),
        DEEP_SCHEMAS.with(Cell::get),
        COMPARISONS.with(Cell::get),
    )
}
