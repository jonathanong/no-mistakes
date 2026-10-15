use super::*;
use std::collections::BTreeMap;
use std::rc::Rc;

type Name = (std::path::PathBuf, usize, String);
pub(super) type Dependencies = Vec<(usize, bool)>;

#[derive(Default)]
pub(super) struct GuardReads {
    names: FxHashMap<Rc<Name>, usize>,
    arena: Vec<Rc<Name>>,
    latest: Vec<Option<usize>>,
    ordered: BTreeMap<usize, usize>,
    clock: usize,
}
impl GuardReads {
    pub(super) fn len(&self) -> usize {
        self.clock
    }
    pub(super) fn append<'a>(
        &mut self,
        names: impl Iterator<Item = &'a Name>,
        cache: &ParameterCache,
    ) {
        for name in names {
            #[cfg(feature = "test-instrumentation")]
            cache.record_nullish_work(13, 1);
            let id = if let Some(id) = self.names.get(name) {
                *id
            } else {
                let id = self.arena.len();
                let name = Rc::new(name.clone());
                self.names.insert(Rc::clone(&name), id);
                self.arena.push(name);
                self.latest.push(None);
                #[cfg(feature = "test-instrumentation")]
                {
                    cache.record_nullish_work(12, 1);
                    cache.record_nullish_peak(14, self.arena.len());
                }
                id
            };
            self.read(id, cache);
        }
    }
    pub(super) fn append_ids(
        &mut self,
        ids: impl Iterator<Item = usize>,
        cache: &ParameterCache,
    ) {
        for id in ids {
            self.read(id, cache);
        }
    }
    fn read(&mut self, id: usize, cache: &ParameterCache) {
        let at = self.clock;
        // A legacy Vec could not represent this many events either. Never
        // wrap an active capture boundary into a different execution prefix.
        self.clock = self.clock.checked_add(1).expect("guard read clock exhausted");
        if let Some(previous) = self.latest[id].replace(at) {
            self.ordered.remove(&previous);
            #[cfg(feature = "test-instrumentation")]
            cache.record_nullish_work(11, 1);
        }
        self.ordered.insert(at, id);
        #[cfg(feature = "test-instrumentation")]
        {
            cache.record_nullish_work(4, 1);
            cache.record_nullish_work(11, 1);
            cache.record_nullish_peak(8, self.ordered.len());
        }
        #[cfg(not(feature = "test-instrumentation"))]
        let _ = cache;
    }
    pub(super) fn capture(
        &self,
        start: usize,
        entry: &FxHashSet<Name>,
        cache: &ParameterCache,
    ) -> Dependencies {
        #[cfg(not(feature = "test-instrumentation"))]
        let _ = cache;
        // A name occurs in the legacy suffix iff its LAST read is in it.
        // Earlier active captures remain valid when a later read replaces one.
        self.ordered
            .range(start..)
            .map(|(_, id)| {
                #[cfg(feature = "test-instrumentation")]
                cache.record_nullish_work(7, 1);
                (*id, entry.contains(self.arena[*id].as_ref()))
            })
            .collect()
    }
    pub(super) fn matches(
        &self,
        dependencies: &Dependencies,
        visiting: &FxHashSet<Name>,
        cache: &ParameterCache,
    ) -> bool {
        for (id, active) in dependencies {
            #[cfg(feature = "test-instrumentation")]
            cache.record_nullish_work(5, 1);
            if visiting.contains(self.arena[*id].as_ref()) != *active {
                #[cfg(feature = "test-instrumentation")]
                cache.record_nullish_work(10, 1);
                return false;
            }
        }
        #[cfg(not(feature = "test-instrumentation"))]
        let _ = cache;
        true
    }
}

#[cfg(test)]
#[path = "nullish_guards_tests.rs"]
mod tests;
