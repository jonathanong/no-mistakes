fn resource_resolution_key(
    root: &Path,
    consumer: &Path,
    call: &ResourceCall,
) -> ResourceResolutionKey {
    match call.kind {
        ResourceCallKind::ReadFile | ResourceCallKind::ReadFileSync => {
            ResourceResolutionKey::Exact(resolve_resource_path(root, consumer, &call.path))
        }
        ResourceCallKind::ReadDirectory | ResourceCallKind::ReadDirectorySync => {
            ResourceResolutionKey::Directory(resolve_resource_path(root, consumer, &call.path))
        }
        ResourceCallKind::Glob | ResourceCallKind::GlobSync => {
            let cwd = call
                .cwd
                .as_ref()
                .map(|cwd| resolve_resource_path(root, consumer, cwd))
                .unwrap_or_else(|| root.to_path_buf());
            let pattern = match call.path.base {
                ResourcePathBase::AnalysisRoot => normalize_glob_pattern(&call.path.value),
                ResourcePathBase::SourceModule => resolve_resource_path(root, consumer, &call.path)
                    .to_string_lossy()
                    .replace('\\', "/"),
            };
            ResourceResolutionKey::Glob { cwd, pattern }
        }
    }
}

fn resolve_resource_path(root: &Path, consumer: &Path, path: &ResourcePath) -> PathBuf {
    let raw = path.value.replace('\\', "/");
    let candidate = PathBuf::from(raw);
    let base = match path.base {
        ResourcePathBase::AnalysisRoot => root,
        ResourcePathBase::SourceModule => consumer.parent().unwrap_or(root),
    };
    let resolved = if candidate.is_absolute() {
        candidate
    } else {
        base.join(candidate)
    };
    crate::codebase::ts_resolver::normalize_path(&resolved)
}

fn normalize_glob_pattern(pattern: &str) -> String {
    let mut normalized = pattern.replace('\\', "/");
    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped.to_string();
    }
    normalized
}

fn expand_resource_key(
    key: &ResourceResolutionKey,
    candidates: &[PathBuf],
    candidate_set: &HashSet<PathBuf>,
) -> Vec<PathBuf> {
    let mut targets = match key {
        ResourceResolutionKey::Exact(path) => candidate_set
            .contains(path)
            .then(|| path.clone())
            .into_iter()
            .collect(),
        ResourceResolutionKey::Directory(directory) => candidates
            .iter()
            .filter(|candidate| candidate.parent() == Some(directory.as_path()))
            .cloned()
            .collect(),
        ResourceResolutionKey::Glob { cwd, pattern } => glob_targets(candidates, cwd, pattern),
    };
    targets.sort();
    targets.dedup();
    targets
}

fn glob_targets(candidates: &[PathBuf], cwd: &Path, pattern: &str) -> Vec<PathBuf> {
    let mut builder = GlobBuilder::new(pattern);
    builder.literal_separator(true);
    let Ok(glob) = builder.build() else {
        return Vec::new();
    };
    let matcher = glob.compile_matcher();
    let matches_absolute_path = Path::new(pattern).is_absolute();
    candidates
        .iter()
        .filter(|candidate| {
            let target = if matches_absolute_path {
                Some(candidate.as_path())
            } else {
                candidate.strip_prefix(cwd).ok()
            };
            target.is_some_and(|path| matcher.is_match(path.to_string_lossy().replace('\\', "/")))
        })
        .cloned()
        .collect()
}

/// Preserve the lexical path stored in the graph, but never permit a tracked
/// path that resolves through a symlink outside the analysis root to become a
/// resource target. Deleted tracked paths have no canonical target, so their
/// nearest existing ancestor establishes containment for the phantom node.
/// This bounded validation runs once per inventory.
fn safe_resource_candidates(root: &Path, candidates: &[PathBuf]) -> Vec<PathBuf> {
    let Some(canonical_root) = std::fs::canonicalize(root).ok() else {
        return Vec::new();
    };
    let mut safe = candidates
        .par_iter()
        .filter(|candidate| {
            std::fs::canonicalize(candidate)
                .ok()
                .or_else(|| {
                    candidate
                        .ancestors()
                        .skip(1)
                        .find_map(|ancestor| std::fs::canonicalize(ancestor).ok())
                })
                .is_some_and(|target| target.starts_with(&canonical_root))
        })
        .cloned()
        .collect::<Vec<_>>();
    safe.sort();
    safe.dedup();
    safe
}
