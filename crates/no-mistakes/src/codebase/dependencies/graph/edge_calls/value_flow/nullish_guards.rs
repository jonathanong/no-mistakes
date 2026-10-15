use super::*;

type Name = (std::path::PathBuf, usize, String);
pub(super) type Dependencies = Vec<(Name, bool)>;

#[derive(Default)]
pub(super) struct GuardReads(Vec<Name>);
impl GuardReads {
    pub(super) fn len(&self) -> usize {
        self.0.len()
    }
    pub(super) fn append(&mut self, names: impl Iterator<Item = Name>, cache: &ParameterCache) {
        let before = self.0.len();
        self.0.extend(names);
        #[cfg(feature = "test-instrumentation")]
        {
            cache.record_nullish_work(4, self.0.len() - before);
            cache.record_nullish_peak(8, self.0.len());
        }
        #[cfg(not(feature = "test-instrumentation"))]
        let _ = (before, cache);
    }
    pub(super) fn capture(
        &self,
        start: usize,
        entry: &FxHashSet<Name>,
        cache: &ParameterCache,
    ) -> Dependencies {
        let mut names = self.0[start..]
            .iter()
            .cloned()
            .collect::<FxHashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        #[cfg(feature = "test-instrumentation")]
        let mut comparisons = 0;
        names.sort_unstable_by(|left, right| {
            #[cfg(feature = "test-instrumentation")]
            {
                comparisons += 1;
            }
            left.cmp(right)
        });
        #[cfg(feature = "test-instrumentation")]
        cache.record_nullish_work(7, self.0.len() - start + names.len() + comparisons);
        #[cfg(not(feature = "test-instrumentation"))]
        let _ = cache;
        names
            .into_iter()
            .map(|name| {
                let active = entry.contains(&name);
                (name, active)
            })
            .collect()
    }
}
pub(super) fn matches(
    dependencies: &Dependencies,
    visiting: &FxHashSet<Name>,
    cache: &ParameterCache,
) -> bool {
    for (name, active) in dependencies {
        #[cfg(feature = "test-instrumentation")]
        cache.record_nullish_work(5, 1);
        if visiting.contains(name) != *active {
            #[cfg(feature = "test-instrumentation")]
            cache.record_nullish_work(10, 1);
            return false;
        }
    }
    #[cfg(not(feature = "test-instrumentation"))]
    let _ = cache;
    true
}

#[cfg(test)]
#[path = "nullish_guards_tests.rs"]
mod tests;
