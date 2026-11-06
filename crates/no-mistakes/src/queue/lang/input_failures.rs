use crate::codebase::lang_frontends::{
    configured_roots, matching_cluster, LangFrontendConfig, QueueGlobMatchers,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) fn selected_inputs(
    root: &Path,
    files: &[PathBuf],
    config: &LangFrontendConfig,
    globs: &QueueGlobMatchers,
) -> Vec<PathBuf> {
    let mut selected = BTreeSet::new();
    add_language_files(
        root,
        files,
        &config.python_packages,
        "py",
        globs,
        &mut selected,
    );
    add_language_files(root, files, &config.go_modules, "go", globs, &mut selected);
    add_language_files(root, files, &config.rails_apps, "rb", globs, &mut selected);
    if config.php_framework.as_deref().is_some_and(|framework| {
        matches!(
            framework.to_ascii_lowercase().as_str(),
            "laravel" | "symfony"
        )
    }) {
        add_language_files(root, files, &config.php_apps, "php", globs, &mut selected);
    }
    add_go_manifests(root, files, &config.go_modules, &mut selected);
    selected.extend(
        files
            .iter()
            .filter(|path| is_queue_glob_input(root, path, globs))
            .cloned(),
    );
    selected.into_iter().collect()
}

pub(super) fn read_failures(
    selected: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<(PathBuf, String)> {
    selected
        .iter()
        .filter_map(|path| {
            sources.read_path(path).err().map(|error| {
                (
                    path.clone(),
                    format!("failed to read {}: {error}", path.display()),
                )
            })
        })
        .collect()
}

fn add_language_files(
    root: &Path,
    files: &[PathBuf],
    entries: &[String],
    extension: &str,
    globs: &QueueGlobMatchers,
    selected: &mut BTreeSet<PathBuf>,
) {
    let roots = configured_roots(root, entries);
    for path in files {
        if path.extension().and_then(|value| value.to_str()) == Some(extension)
            && roots.iter().any(|scope| path.starts_with(scope))
            && is_queue_glob_input(root, path, globs)
        {
            selected.insert(path.clone());
        }
    }
}

fn add_go_manifests(
    root: &Path,
    files: &[PathBuf],
    modules: &[String],
    selected: &mut BTreeSet<PathBuf>,
) {
    for module_root in configured_roots(root, modules) {
        let has_selected_source = selected.iter().any(|path| path.starts_with(&module_root));
        let manifest = module_root.join("go.mod");
        if has_selected_source && files.contains(&manifest) {
            selected.insert(manifest);
        }
    }
}

fn is_queue_glob_input(root: &Path, path: &Path, globs: &QueueGlobMatchers) -> bool {
    matching_cluster(root, path, &globs.enqueues, globs).is_some()
        || matching_cluster(root, path, &globs.workers, globs).is_some()
}

#[cfg(test)]
mod tests;
