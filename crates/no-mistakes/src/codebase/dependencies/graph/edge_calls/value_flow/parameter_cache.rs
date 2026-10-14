use super::*;
use dashmap::DashMap;
use rustc_hash::FxBuildHasher;
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::thread::ThreadId;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct ParameterKey {
    pub(super) file: std::path::PathBuf,
    pub(super) scope: usize,
    pub(super) name: String,
    pub(super) invocation_read: Option<(std::path::PathBuf, u32, Option<CallableId>, CallableId)>,
    pub(super) initializing: Vec<(std::path::PathBuf, CallableId, usize)>,
}
#[derive(Clone)]
pub(super) struct CachedParameter {
    pub(super) value: FlowValue,
    pub(super) origins: default_origins::Origins,
}
enum Entry {
    Computing(ThreadId),
    Ready(Arc<CachedParameter>),
}
type Owner = (std::path::PathBuf, CallableId);
#[derive(Default)]
struct State {
    entries: FxHashMap<ParameterKey, Entry>,
    factory_agreement: FxHashMap<Owner, bool>,
    waiting: FxHashMap<ThreadId, ThreadId>,
    computations: usize,
    reverse_dependencies: FxHashMap<Owner, FxHashSet<Owner>>,
}
type TypeOrigins = DashMap<
    (std::path::PathBuf, String),
    Arc<OnceLock<(String, String)>>,
    FxBuildHasher,
>;
#[derive(Default)]
pub(in super::super) struct ParameterCache {
    state: Mutex<State>,
    ready: Condvar,
    type_origins: OnceLock<TypeOrigins>,
    #[cfg(feature = "test-instrumentation")]
    origin_nodes: std::sync::atomic::AtomicUsize,
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
    pub(super) fn type_origin(
        &self,
        key: (std::path::PathBuf, String),
        resolve: impl FnOnce() -> (String, String),
    ) -> (String, String) {
        // Keep cold shard construction small; resolutions run outside guards,
        // so the shard count does not cap concurrent independent resolutions.
        let origins = self
            .type_origins
            .get_or_init(|| DashMap::with_hasher_and_shard_amount(FxBuildHasher, 8));
        if let Some(value) = origins.get(&key).and_then(|slot| slot.get().cloned()) {
            return value;
        }
        let slot = Arc::clone(origins.entry(key).or_default().value());
        // Release the shard guard before walking prepared type re-exports. Even
        // unrelated keys in the same shard must be able to resolve concurrently.
        slot.get_or_init(resolve).clone()
    }
    pub(super) fn origin_node(&self, kind: OriginKind, height: usize) -> Arc<OriginNode> {
        #[cfg(feature = "test-instrumentation")]
        self.origin_nodes
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Arc::new(OriginNode::new(kind, height))
    }
    #[cfg(feature = "test-instrumentation")]
    pub(in super::super) fn origin_nodes(&self) -> usize {
        self.origin_nodes.load(std::sync::atomic::Ordering::Relaxed)
    }

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
    pub(super) fn factory_agrees(&self, owner: Owner, compute: impl FnOnce() -> bool) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        *state.factory_agreement.entry(owner).or_insert_with(compute)
    }
    pub(super) fn clear(&self) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.entries.clear();
        state.factory_agreement.clear();
    }
    #[cfg(feature = "test-instrumentation")]
    pub(in super::super) fn computations(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .computations
    }
    fn publish(
        &self,
        key: ParameterKey,
        value: Arc<CachedParameter>,
        owner: Option<Owner>,
        dependencies: FxHashSet<Owner>,
    ) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(owner) = owner {
            state.record_dependencies(owner, dependencies);
        }
        state.entries.insert(key, Entry::Ready(value));
        drop(state);
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
    pub(super) fn publish(
        mut self,
        value: Arc<CachedParameter>,
        owner: Owner,
        dependencies: FxHashSet<Owner>,
    ) {
        self.cache.publish(
            self.key.take().expect("cache lease"),
            value,
            Some(owner),
            dependencies,
        );
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
                }),
                None,
                fx_set(),
            );
        }
    }
}

#[path = "parameter_dependencies.rs"]
mod dependencies;

#[cfg(test)]
#[path = "parameter_cache_tests.rs"]
mod tests;
