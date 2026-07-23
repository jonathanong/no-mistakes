use crate::codebase::ts_resources::{ResourceCallKind, ResourcePath, ResourcePathBase};
use oxc_ast::ast::Program;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Keep literal resource reads from a resolved setup as deletion triggers.
/// The graph cannot recreate an edge when the resource itself is gone.
pub(super) fn paths(
    program: &Program<'_>,
    source: &str,
    project_root: &Path,
    setup_path: &Path,
) -> BTreeSet<PathBuf> {
    crate::codebase::ts_resources::extract(program, source)
        .calls
        .into_iter()
        .filter(|call| {
            matches!(
                call.kind,
                ResourceCallKind::ReadFile | ResourceCallKind::ReadFileSync
            )
        })
        .map(|call| resolve(project_root, setup_path, call.path))
        .filter(|candidate| candidate.starts_with(project_root))
        .collect()
}

fn resolve(project_root: &Path, setup_path: &Path, path: ResourcePath) -> PathBuf {
    let candidate = PathBuf::from(path.value.replace('\\', "/"));
    let base = match path.base {
        ResourcePathBase::AnalysisRoot => project_root,
        ResourcePathBase::SourceModule => setup_path.parent().unwrap_or(project_root),
    };
    crate::codebase::ts_resolver::normalize_path(&if candidate.is_absolute() {
        candidate
    } else {
        base.join(candidate)
    })
}

#[cfg(test)]
mod tests;
