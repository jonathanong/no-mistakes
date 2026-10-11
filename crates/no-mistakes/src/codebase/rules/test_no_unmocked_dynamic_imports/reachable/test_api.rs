use super::*;

pub(in super::super) fn collect(
    ctx: ReachableContext<'_>,
    test_file: &Path,
    mocks: &HashSet<PathBuf>,
    dependency_cache: &DashMap<PathBuf, Arc<Vec<PathBuf>>>,
) -> Result<ReachableResult> {
    deferred::collect(ctx, test_file, mocks, dependency_cache, false)
}
