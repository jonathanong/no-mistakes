use crate::codebase::ts_resources::{
    ResourceCall, ResourceCallKind, ResourcePath, ResourcePathBase,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum ResourceResolutionKey {
    Exact(PathBuf),
    Directory(PathBuf),
    Glob { cwd: PathBuf, pattern: String },
}

#[derive(Debug, Clone)]
struct ResolvedResourceCall {
    consumer: PathBuf,
    key: ResourceResolutionKey,
    site: ResourceCallSite,
}

/// Produce canonical consumer → resource edges using only the prepared tracked
/// inventory. The inventory is never walked from a call site: directory and
/// glob expansions are memoized for this build and repeated calls reuse them.
fn collect_resource_edges(
    root: &Path,
    files: &[PathBuf],
    facts: &dyn TsFactLookup,
    resource_candidates: &[PathBuf],
) -> (Vec<Edge>, ResourceEdgeDetails, Vec<ResourceGraphDiagnostic>) {
    // Parsing already happened when collecting TS facts.  Each consumer can be
    // filtered independently, so keep this collection parallel and sort below
    // before constructing the canonical graph output.
    let (mut calls, mut diagnostics): (Vec<_>, Vec<_>) = files
        .par_iter()
        .map(|consumer| {
            let Some(file_facts) = facts.get_ts_facts(consumer) else {
                return (Vec::new(), Vec::new());
            };
            let reachable = reachable_function_scopes(file_facts);
            let diagnostics = file_facts
                .resource_diagnostics
                .iter()
                .filter(|diagnostic| {
                    resource_diagnostic_is_reachable(diagnostic, file_facts, &reachable)
                })
                .map(|diagnostic| ResourceGraphDiagnostic {
                    consumer: consumer.clone(),
                    kind: diagnostic.kind,
                    line: diagnostic.line,
                })
                .collect();
            let calls = file_facts
                .resource_calls
                .iter()
                .filter(|call| resource_is_reachable(call, file_facts, &reachable))
                .map(|call| ResolvedResourceCall {
                    consumer: consumer.clone(),
                    key: resource_resolution_key(root, consumer, call),
                    site: ResourceCallSite {
                        call_kind: match call.kind {
                            ResourceCallKind::ReadFile => "read-file",
                            ResourceCallKind::ReadFileSync => "read-file-sync",
                            ResourceCallKind::ReadDirectory => "read-directory",
                            ResourceCallKind::ReadDirectorySync => "read-directory-sync",
                            ResourceCallKind::Glob => "glob",
                            ResourceCallKind::GlobSync => "glob-sync",
                        }
                        .to_string(),
                        line: call.line,
                    },
                })
                .collect();
            (calls, diagnostics)
        })
        .reduce(
            || (Vec::new(), Vec::new()),
            |(mut calls, mut diagnostics), (mut next_calls, mut next_diagnostics)| {
                calls.append(&mut next_calls);
                diagnostics.append(&mut next_diagnostics);
                (calls, diagnostics)
            },
        );
    calls.sort_by(|left, right| {
        (&left.consumer, &left.key, &left.site).cmp(&(&right.consumer, &right.key, &right.site))
    });
    calls.dedup_by(|left, right| {
        left.consumer == right.consumer && left.key == right.key && left.site == right.site
    });
    diagnostics.sort();
    diagnostics.dedup();
    // Dynamic resource diagnostics do not need the tracked resource inventory.
    // Avoid canonicalizing every tracked file when there are no literal calls to
    // expand, which is the common case in projects that only use dynamic paths.
    if calls.is_empty() {
        return (Vec::new(), HashMap::new(), diagnostics);
    }
    let candidates = safe_resource_candidates(root, resource_candidates);
    let candidate_set: HashSet<PathBuf> = candidates.iter().cloned().collect();

    // Expand each unique static resource once.  This is intentionally derived
    // from the prepared candidate list rather than walking the filesystem from
    // every call site, and independent matcher expansions run in parallel.
    let mut unique_keys = calls
        .iter()
        .map(|call| call.key.clone())
        .collect::<Vec<_>>();
    unique_keys.sort();
    unique_keys.dedup();
    let expansion_cache: HashMap<ResourceResolutionKey, Vec<PathBuf>> = unique_keys
        .into_par_iter()
        .map(|key| {
            let targets = expand_resource_key(&key, &candidates, &candidate_set);
            (key, targets)
        })
        .collect();
    let mut edges = Vec::new();
    let mut details: ResourceEdgeDetails = HashMap::new();
    for call in calls {
        let targets = expansion_cache
            .get(&call.key)
            .expect("every collected resource key has a cached expansion");
        for target in targets {
            edges.push((
                NodeId::File(call.consumer.clone()),
                NodeId::File(target.clone()),
                EdgeKind::Resource,
            ));
            details
                .entry((call.consumer.clone(), target.clone()))
                .or_default()
                .push(call.site.clone());
        }
    }
    for sites in details.values_mut() {
        sites.sort();
        sites.dedup();
    }
    edges.sort_by(|left, right| {
        (&left.0, &left.1, left.2.sort_key()).cmp(&(
            &right.0,
            &right.1,
            right.2.sort_key(),
        ))
    });
    edges.dedup();
    (edges, details, diagnostics)
}
