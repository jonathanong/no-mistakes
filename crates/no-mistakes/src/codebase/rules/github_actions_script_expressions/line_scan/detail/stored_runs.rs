use std::cell::Cell;

thread_local! {
    static STORED_RUN_BLOCKS: Cell<usize> = const { Cell::new(0) };
}

/// One increment per run block moved into `scanner.runs`.
pub(super) fn note_stored_run_block() {
    STORED_RUN_BLOCKS.with(|count| count.set(count.get() + 1));
}

pub(super) fn stored_run_blocks() -> usize {
    STORED_RUN_BLOCKS.with(|count| count.get())
}
