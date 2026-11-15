use std::cell::Cell;

thread_local! {
    static RELATED_FILE_COLLECTIONS: Cell<u32> = const { Cell::new(0) };
}

pub(in super::super) fn take_related_file_collections() -> u32 {
    RELATED_FILE_COLLECTIONS.with(|count| count.replace(0))
}

pub(super) fn note_related_file_collection() {
    RELATED_FILE_COLLECTIONS.with(|count| count.set(count.get() + 1));
}
