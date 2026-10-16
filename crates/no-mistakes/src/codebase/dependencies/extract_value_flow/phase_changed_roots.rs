use super::*;
/// Keep the first owned root inline; membership falls back to FxHashSet when
/// a second distinct root is recorded, without changing phase lifetimes.
#[cfg_attr(not(all(test, feature = "test-instrumentation")), derive(Default))]
pub(super) enum ChangedRoots {
    #[cfg_attr(not(all(test, feature = "test-instrumentation")), default)]
    Empty,
    One(String),
    Many(FxHashSet<String>),
    #[cfg(all(test, feature = "test-instrumentation"))]
    Legacy(phase_changed_roots_tests::LegacyRoots),
}
#[cfg(all(test, feature = "test-instrumentation"))]
impl Default for ChangedRoots {
    fn default() -> Self {
        #[cfg(all(test, feature = "test-instrumentation"))]
        if phase_changed_roots_tests::LEGACY.with(std::cell::Cell::get) {
            return Self::Legacy(Default::default());
        }
        #[cfg(all(test, feature = "test-instrumentation"))]
        phase_changed_roots_tests::record(0);
        Self::Empty
    }
}
impl ChangedRoots {
    #[cfg(all(test, feature = "test-instrumentation"))]
    pub(super) fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::One(_) => 1,
            Self::Many(names) => names.len(),
            Self::Legacy(names) => names.len(),
        }
    }
    pub(super) fn contains(&self, name: &str) -> bool {
        #[cfg(all(test, feature = "test-instrumentation"))]
        phase_changed_roots_tests::record(1);
        let found = match self {
            Self::Empty => false,
            Self::One(first) => first == name,
            Self::Many(names) => names.contains(name),
            #[cfg(all(test, feature = "test-instrumentation"))]
            Self::Legacy(names) => names.contains(name),
        };
        #[cfg(all(test, feature = "test-instrumentation"))]
        if found {
            phase_changed_roots_tests::record(9);
        }
        found
    }
    pub(super) fn insert(&mut self, name: String) {
        #[cfg(all(test, feature = "test-instrumentation"))]
        phase_changed_roots_tests::record(2);
        match self {
            Self::Empty => *self = Self::One(name),
            Self::One(first) if first == &name => {}
            Self::One(first) => {
                let first = std::mem::take(first);
                let mut names = fx_set();
                names.insert(first);
                names.insert(name);
                #[cfg(all(test, feature = "test-instrumentation"))]
                phase_changed_roots_tests::record(4);
                *self = Self::Many(names);
            }
            Self::Many(names) => {
                #[cfg(all(test, feature = "test-instrumentation"))]
                let capacity = names.capacity();
                names.insert(name);
                #[cfg(all(test, feature = "test-instrumentation"))]
                if names.capacity() > capacity {
                    phase_changed_roots_tests::record(4);
                }
            }
            #[cfg(all(test, feature = "test-instrumentation"))]
            Self::Legacy(names) => names.insert(name),
        }
    }
}
const _: () =
    assert!(size_of::<Option<ChangedRoots>>() <= size_of::<Option<FxHashSet<String>>>() + 8);
#[cfg(test)]
#[path = "phase_changed_roots_unit_tests.rs"]
mod tests;
