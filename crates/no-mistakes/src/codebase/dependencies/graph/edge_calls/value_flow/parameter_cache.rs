use super::*;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::ThreadId;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct ParameterKey {
    pub(super) file: std::path::PathBuf,
    pub(super) scope: usize,
    pub(super) name: String,
    pub(super) initializing: Vec<(std::path::PathBuf, CallableId, usize)>,
}
#[derive(Clone)]
pub(super) struct CachedParameter {
    pub(super) value: FlowValue,
    pub(super) origins: Option<Arc<FxHashSet<DefaultOriginUse>>>,
    pub(super) dependencies: FxHashSet<(std::path::PathBuf, CallableId)>,
}
enum Entry {
    Computing(ThreadId),
    Ready(Arc<CachedParameter>),
}
#[derive(Default)]
struct State {
    entries: FxHashMap<ParameterKey, Entry>,
    waiting: FxHashMap<ThreadId, ThreadId>,
    computations: usize,
}
#[derive(Default)]
pub(in super::super) struct ParameterCache {
    state: Mutex<State>,
    ready: Condvar,
}
pub(super) enum CacheAccess<'a> {
    Ready(Arc<CachedParameter>),
    Compute(Lease<'a>),
    Cycle,
}
pub(super) struct Lease<'a> {
    cache: &'a ParameterCache,
    key: Option<ParameterKey>,
}
impl ParameterCache {
    pub(super) fn acquire(&self, key: ParameterKey) -> CacheAccess<'_> {
        let thread = std::thread::current().id();
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        loop {
            match state.entries.get(&key) {
                Some(Entry::Ready(value)) => return CacheAccess::Ready(Arc::clone(value)),
                Some(Entry::Computing(owner)) => {
                    let owner = *owner;
                    if owner == thread || waits_for(&state.waiting, owner, thread) {
                        return CacheAccess::Cycle;
                    }
                    state.waiting.insert(thread, owner);
                    state = self
                        .ready
                        .wait(state)
                        .unwrap_or_else(|error| error.into_inner());
                    state.waiting.remove(&thread);
                }
                None => {
                    state.entries.insert(key.clone(), Entry::Computing(thread));
                    state.computations += 1;
                    return CacheAccess::Compute(Lease {
                        cache: self,
                        key: Some(key),
                    });
                }
            }
        }
    }
    pub(super) fn clear(&self) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .entries
            .clear();
    }
    #[cfg(feature = "test-instrumentation")]
    pub(in super::super) fn computations(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .computations
    }
    fn publish(&self, key: ParameterKey, value: Arc<CachedParameter>) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .entries
            .insert(key, Entry::Ready(value));
        self.ready.notify_all();
    }
}
fn waits_for(
    waiting: &FxHashMap<ThreadId, ThreadId>,
    mut owner: ThreadId,
    thread: ThreadId,
) -> bool {
    let mut visited = fx_set();
    while visited.insert(owner) {
        if owner == thread {
            return true;
        }
        let Some(next) = waiting.get(&owner) else {
            return false;
        };
        owner = *next;
    }
    false
}
impl Lease<'_> {
    pub(super) fn publish(mut self, value: Arc<CachedParameter>) {
        self.cache
            .publish(self.key.take().expect("cache lease"), value);
    }
}
impl Drop for Lease<'_> {
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            self.cache.publish(
                key,
                Arc::new(CachedParameter {
                    value: FlowValue::Unknown,
                    origins: None,
                    dependencies: fx_set(),
                }),
            );
        }
    }
}
