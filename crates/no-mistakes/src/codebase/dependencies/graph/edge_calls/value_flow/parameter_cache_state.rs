#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct ParameterKey {
    pub(super) file: std::path::PathBuf,
    pub(super) scope: usize,
    pub(super) name: String,
    pub(super) invocation_read: Option<default_origins::InvocationRead>,
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
type TypeOrigins =
    DashMap<(std::path::PathBuf, String), Arc<OnceLock<(String, String)>>, FxBuildHasher>;

impl ParameterCache {
    pub(super) fn for_callback_ownership() -> Self {
        Self {
            ownership_callbacks: true,
            ..Default::default()
        }
    }
}
