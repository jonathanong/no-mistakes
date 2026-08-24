use std::path::{Path, PathBuf};

pub(crate) fn global_config_trigger(
    root: &Path,
    changed_files: &[PathBuf],
    framework: Option<crate::tests::TestFramework>,
    prepared: &super::super::prepared_plan::PreparedTestPlanRequest,
) -> Option<(String, PathBuf)> {
    changed_files.iter().find_map(|file| {
        if prepared.is_dependency_only_manifest(file, framework) {
            return None;
        }
        let relative_changed = super::relative_path(root, file);
        is_global_config_path(root, file, &relative_changed).then(|| {
            (
                format!("Global configuration file changed: {relative_changed}"),
                file.clone(),
            )
        })
    })
}

/// Framework plans compare `.no-mistakes.yml`/`.yaml` at both endpoints, so
/// an unrelated framework's formatting-only edit does not invalidate them.
/// All other historical global configuration triggers retain their existing
/// unconditional behavior.
pub(super) fn excluding_v2_config(
    root: &Path,
    changed_files: &[PathBuf],
    framework: Option<crate::tests::TestFramework>,
    prepared: &super::super::prepared_plan::PreparedTestPlanRequest,
) -> Option<(String, PathBuf)> {
    changed_files.iter().find_map(|file| {
        if prepared.is_dependency_only_manifest(file, framework) {
            return None;
        }
        let relative_changed = super::relative_path(root, file);
        (!matches!(
            relative_changed.as_str(),
            ".no-mistakes.yml" | ".no-mistakes.yaml"
        ) && is_global_config_path(root, file, &relative_changed))
        .then(|| {
            (
                format!("Global configuration file changed: {relative_changed}"),
                file.clone(),
            )
        })
    })
}

fn is_global_config_path(root: &Path, absolute: &Path, relative: &str) -> bool {
    if matches!(
        relative,
        "package.json"
            | "pnpm-workspace.yaml"
            | "tsconfig.json"
            | ".no-mistakes.yml"
            | ".no-mistakes.yaml"
    ) {
        return true;
    }

    let Some(name) = absolute.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if !matches!(
        name,
        "next.config.js"
            | "next.config.mjs"
            | "next.config.ts"
            | "next.config.mts"
            | "proxy.js"
            | "proxy.mjs"
            | "proxy.ts"
            | "proxy.mts"
            | "middleware.js"
            | "middleware.mjs"
            | "middleware.ts"
            | "middleware.mts"
    ) {
        return false;
    }

    absolute
        .parent()
        .is_some_and(|parent| parent == root || super::next_project_root(parent))
}

pub(super) fn discover_all_tests_from_prepared(
    prepared: &super::super::prepared_plan::PreparedTestPlanRequest,
) -> Vec<PathBuf> {
    no_mistakes::codebase::ts_source::discover_files_from_visible(
        &prepared.root,
        &prepared.config.filesystem.skip_directories,
        prepared.root_visible_paths(),
    )
    .into_iter()
    .filter(|file| {
        prepared
            .visible_paths
            .classification_for(&prepared.root, file)
            .is_some_and(|classification| classification.target_is_file())
    })
    .filter(|file| prepared.test_filter().is_match(&prepared.root, file))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pnpm_workspace_configuration_is_a_root_scoped_broad_trigger() {
        let root = Path::new("/repo");
        assert!(is_global_config_path(
            root,
            &root.join("pnpm-workspace.yaml"),
            "pnpm-workspace.yaml"
        ));
        assert!(!is_global_config_path(
            root,
            &root.join("packages/app/pnpm-workspace.yaml"),
            "packages/app/pnpm-workspace.yaml"
        ));
    }
}
