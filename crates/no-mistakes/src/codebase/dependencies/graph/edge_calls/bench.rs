/// Criterion adapter. Returns a size so the compiler cannot drop the index.
#[doc(hidden)]
pub fn benchmark_construct_callable_file_index(
    facts: &crate::codebase::ts_source::facts::TsFileFacts,
) -> usize {
    let index = CallableFileIndex::from_facts(facts);
    index
        .aliases
        .values()
        .map(|names| names.len())
        .sum::<usize>()
        + index
            .callable_bindings
            .values()
            .map(|names| names.len())
            .sum::<usize>()
        + index
            .class_bindings
            .values()
            .map(|names| names.len())
            .sum::<usize>()
        + index.imported.len()
        + index.exported.len()
}

/// Criterion adapter for per-file call-site membership after the sites are sorted.
#[doc(hidden)]
pub fn benchmark_probe_call_site_files(file_count: usize) -> usize {
    let mut sites = Vec::with_capacity(file_count);
    for index in 0..file_count {
        sites.push(ResolvedCallSite {
            target_node: None,
            file: std::path::PathBuf::from(format!("/{index}.ts")),
            caller: None,
            caller_id: None,
            line: 1,
            offset: 0,
            invocation: crate::codebase::dependencies::extract::InvocationKind::Call,
            source_callee: "f".to_string(),
            target: ResolvedCallTarget::Unknown,
        });
    }
    sites.sort_by(|left, right| left.file.cmp(&right.file));
    let index = index_sorted_call_sites_by_file(&sites);
    (0..file_count)
        .filter(|file| index.contains_key(std::path::Path::new(&format!("/{file}.ts"))))
        .count()
}

/// Prepared callable index for timing lookups independently of construction.
#[doc(hidden)]
pub struct BenchmarkCallableIndex {
    index: CallableFileIndex,
}

/// Build the same per-file index used by call and import reachability.
#[doc(hidden)]
pub fn benchmark_prepare_callable_index(facts: &TsFileFacts) -> BenchmarkCallableIndex {
    BenchmarkCallableIndex {
        index: CallableFileIndex::from_facts(facts),
    }
}

impl BenchmarkCallableIndex {
    pub fn local_id(&self, scope: usize, name: &str) -> Option<u32> {
        self.index
            .resolve_local_callable_id(Some(scope), name)
            .map(|id| id.0)
    }

    pub fn alias_resolves(&self, scope: usize, name: &str) -> bool {
        self.index
            .resolve_alias(
                None,
                Some(scope),
                name,
                u32::MAX,
                None,
                InvocationKind::Call,
            )
            .is_some()
    }

    pub fn binding_live(&self, scope: usize, name: &str) -> bool {
        self.index
            .target_binding_live(scope, name, u32::MAX, Some(scope), None)
    }

    pub fn class_id(&self, scope: usize, name: &str) -> Option<u32> {
        self.index
            .resolve_class_binding(Some(scope), name, InvocationKind::Call)
            .and_then(|callee| callee.callable_id)
            .map(|id| id.0)
    }
}
