use super::*;

thread_local! {
    static ORIGINAL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Captured once in the final request-owned CallPass cache, before Rayon refinement.
pub(super) struct Mode(bool);
impl Default for Mode {
    fn default() -> Self {
        Self(ORIGINAL.get())
    }
}
impl Mode {
    pub(super) fn original(&self) -> bool {
        self.0
    }
}

pub fn with_original_admission<T>(original: bool, run: impl FnOnce() -> T) -> T {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            ORIGINAL.set(self.0);
        }
    }
    let _restore = Restore(ORIGINAL.replace(original));
    run()
}

pub(super) fn capture_edges(snapshot: &mut ReplacementProbeSnapshot, edges: &[Edge]) {
    // Preserve original order and duplicate entries; tests compare multiplicity separately.
    snapshot.admission_raw_edges = edges.iter().map(|edge| format!("{edge:?}")).collect();
}

#[cfg(test)]
#[path = "admission_mode_tests.rs"]
mod tests;
