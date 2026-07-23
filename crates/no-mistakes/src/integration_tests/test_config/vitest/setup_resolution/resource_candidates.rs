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
mod tests {
    use super::*;

    #[test]
    fn resource_candidates_follow_graph_path_bases() {
        let root = Path::new("/repo");
        let setup = Path::new("/repo/packages/unit/setup.ts");
        for (path, expected) in [
            (
                ResourcePath {
                    value: "resources/data.json".to_string(),
                    base: ResourcePathBase::AnalysisRoot,
                },
                PathBuf::from("/repo/resources/data.json"),
            ),
            (
                ResourcePath {
                    value: "./local.json".to_string(),
                    base: ResourcePathBase::SourceModule,
                },
                PathBuf::from("/repo/packages/unit/local.json"),
            ),
            (
                ResourcePath {
                    value: "/tmp/absolute.json".to_string(),
                    base: ResourcePathBase::AnalysisRoot,
                },
                PathBuf::from("/tmp/absolute.json"),
            ),
        ] {
            assert_eq!(resolve(root, setup, path), expected);
        }
    }
}
