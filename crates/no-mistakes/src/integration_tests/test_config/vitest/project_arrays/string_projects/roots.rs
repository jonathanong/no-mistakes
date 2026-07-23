use super::{is_vitest_project_config, slash_path, Ctx};
use crate::codebase::ts_resolver::ImportResolution;
use globset::GlobBuilder;
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

/// Vitest project strings may point at a folder without a local config. Keep
/// that folder as a default project when it is explicitly matched by the
/// visible project glob, instead of silently dropping its tests.
pub(in crate::integration_tests::test_config::vitest::project_arrays) fn string_project_roots(
    specifier: &str,
    ctx: &Ctx<'_, '_>,
) -> Vec<PathBuf> {
    string_project_roots_with_resolver(specifier, ctx.path, ctx.resolver)
}

pub(in crate::integration_tests::test_config::vitest) fn string_project_roots_with_resolver(
    specifier: &str,
    declaration_path: &Path,
    resolver: &dyn ImportResolution,
) -> Vec<PathBuf> {
    let Some(visible) = resolver.visible_files() else {
        return Vec::new();
    };
    let base = declaration_path.parent().unwrap_or(Path::new("."));
    let pattern = crate::codebase::ts_resolver::normalize_path(
        &base.join(specifier.trim_start_matches("./")),
    );
    let glob = specifier.contains(['*', '?', '[', '{']).then(|| {
        GlobBuilder::new(&slash_path(&pattern))
            .literal_separator(true)
            .build()
            .map(|glob| glob.compile_matcher())
    });
    let mut roots = BTreeSet::new();
    for path in visible {
        let mut parent = path.parent();
        while let Some(root) = parent {
            if root == base.parent().unwrap_or(base) {
                break;
            }
            let matches = match &glob {
                Some(Ok(glob)) => glob.is_match(slash_path(root)),
                Some(Err(_)) => false,
                None => root == pattern,
            };
            if matches && !has_project_config(root, visible) {
                roots.insert(root.to_path_buf());
            }
            parent = root.parent();
        }
    }
    roots.into_iter().collect()
}

fn has_project_config(root: &Path, visible: &HashSet<PathBuf>) -> bool {
    visible
        .iter()
        .any(|path| path.parent() == Some(root) && is_vitest_project_config(path))
}
