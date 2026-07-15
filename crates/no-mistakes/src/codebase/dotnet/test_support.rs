use std::path::{Path, PathBuf};

thread_local! {
    static FACT_COLLECTION_COUNT: std::cell::RefCell<Option<(PathBuf, usize)>> = const { std::cell::RefCell::new(None) };
}

pub fn begin_fact_collection_count(root: &Path) {
    FACT_COLLECTION_COUNT.with(|state| *state.borrow_mut() = Some((root.to_path_buf(), 0)));
}

pub(super) fn record_fact_collection(root: &Path) {
    FACT_COLLECTION_COUNT.with(|state| {
        if let Some((session_root, count)) = state.borrow_mut().as_mut() {
            if session_root == root {
                *count += 1;
            }
        }
    });
}

pub fn finish_fact_collection_count(root: &Path) -> usize {
    FACT_COLLECTION_COUNT.with(|state| {
        let (session_root, count) = state
            .borrow_mut()
            .take()
            .expect("Dotnet fact collection count was not started");
        assert_eq!(session_root, root);
        count
    })
}
