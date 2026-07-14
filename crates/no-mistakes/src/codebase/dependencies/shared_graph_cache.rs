#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct GraphFileUniverseKey {
    generation: u64,
    paths: std::sync::Arc<[PathBuf]>,
}

impl GraphFileUniverseKey {
    fn new(files: &graph::GraphFiles, generation: u64) -> Self {
        let mut paths = files.all().to_vec();
        paths.sort();
        paths.dedup();
        Self {
            generation,
            paths: paths.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct EffectiveGraphPlanKey {
    plan: graph::GraphBuildPlan,
    files: GraphFileUniverseKey,
}

impl EffectiveGraphPlanKey {
    fn new(plan: graph::GraphBuildPlan, files: &graph::GraphFiles, generation: u64) -> Self {
        Self {
            plan,
            files: GraphFileUniverseKey::new(files, generation),
        }
    }
}

type SharedBuildResult<V> =
    std::sync::Arc<std::sync::OnceLock<Result<std::sync::Arc<V>, std::sync::Arc<str>>>>;

struct SharedBuildCache<K, V> {
    entries: std::sync::Mutex<HashMap<K, SharedBuildResult<V>>>,
    builds: std::sync::atomic::AtomicUsize,
}

impl<K, V> Default for SharedBuildCache<K, V> {
    fn default() -> Self {
        Self {
            entries: std::sync::Mutex::new(HashMap::new()),
            builds: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

impl<K: Eq + std::hash::Hash, V> SharedBuildCache<K, V> {
    fn get_or_build(&self, key: K, build: impl FnOnce() -> Result<V>) -> Result<std::sync::Arc<V>> {
        let cell = {
            let mut entries = self.entries.lock().expect("shared build cache is poisoned");
            entries
                .entry(key)
                .or_insert_with(|| std::sync::Arc::new(std::sync::OnceLock::new()))
                .clone()
        };
        let result = cell.get_or_init(|| {
            self.builds
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            build()
                .map(std::sync::Arc::new)
                .map_err(|error| std::sync::Arc::<str>::from(format!("{error:#}")))
        });
        result.clone().map_err(|message| anyhow::anyhow!(message))
    }

    fn clear(&self) {
        self.entries
            .lock()
            .expect("shared build cache is poisoned")
            .clear();
    }

    fn build_count(&self) -> usize {
        self.builds.load(std::sync::atomic::Ordering::Relaxed)
    }
}

struct CanonicalGraphBuild<'a> {
    root: &'a Path,
    tsconfig: &'a TsConfig,
    plan: graph::GraphBuildPlan,
    graph_files: &'a graph::GraphFiles,
    config_path: Option<&'a Path>,
    prepared_graph: &'a graph::PreparedGraphConfig,
    facts: Option<&'a dyn graph::TsFactLookup>,
    import_resolution_cache: &'a crate::codebase::ts_resolver::ImportResolutionCache,
}

fn build_canonical_graph(input: CanonicalGraphBuild<'_>) -> Result<graph::DepGraph> {
    graph::DepGraph::build_with_plan_files_prepared_config_facts_and_resolution_cache(
        graph::PreparedGraphBuild {
            root: input.root,
            tsconfig: input.tsconfig,
            plan: input.plan,
            graph_files: input.graph_files,
            config_path: input.config_path,
            prepared: input.prepared_graph,
            facts: input.facts,
            import_resolution_cache: Some(input.import_resolution_cache),
        },
    )
}
