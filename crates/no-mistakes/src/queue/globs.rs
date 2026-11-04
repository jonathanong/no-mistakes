use crate::queue::extract::FileFacts;
use crate::queue::extract_model::LocalCheck;
use globset::{GlobBuilder, GlobSet};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(super) fn record_empty_globs(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    sources: &crate::codebase::ts_source::SourceStore,
    files: &[PathBuf],
    facts: &mut HashMap<PathBuf, FileFacts>,
) {
    let relative = files
        .iter()
        .filter_map(|path| path.strip_prefix(root).ok().map(Path::to_path_buf))
        .collect::<Vec<_>>();
    for (name, project) in &config.projects {
        for (field, globs) in [
            ("enqueues", &project.queues.enqueues),
            ("workers", &project.queues.workers),
        ] {
            for (index, configured) in globs.iter().enumerate() {
                let glob = prefixed_one(&project.root, configured);
                if !glob_matches(&glob, &relative) {
                    record_empty_glob(root, config, sources, name, field, index, &glob, facts);
                }
            }
        }
    }
}

fn record_empty_glob(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    sources: &crate::codebase::ts_source::SourceStore,
    project: &str,
    field: &str,
    index: usize,
    glob: &str,
    facts: &mut HashMap<PathBuf, FileFacts>,
) {
    let key = crate::config::source_locations::ConfigPath::default()
        .key("projects")
        .key(project)
        .key("queues")
        .key(field)
        .index(index);
    let (path, local) = config
        .config_provenance
        .origin(&key)
        .map(|(path, local)| (path.to_path_buf(), Some(local)))
        .unwrap_or_else(|| {
            (
                config
                    .loaded_config_path
                    .0
                    .clone()
                    .unwrap_or_else(|| root.join(".no-mistakes.yml")),
                None,
            )
        });
    let line = local
        .and_then(|local| sources.config_locations(&path)?.line(&local))
        .unwrap_or(1);
    facts.entry(path).or_default().checks.push(LocalCheck {
        line,
        kind: "empty-glob".to_string(),
        job: None,
        message: format!("queue glob '{glob}' matches no files"),
    });
}

fn prefixed_one(root: &Option<String>, glob: &str) -> String {
    let prefix = root
        .as_deref()
        .map(str::trim)
        .map(|root| root.trim_start_matches("./").trim_end_matches('/'))
        .filter(|root| !root.is_empty() && *root != ".");
    let glob = glob.trim_start_matches("./");
    match prefix {
        Some(root) if glob == root || glob.starts_with(&format!("{root}/")) => glob.to_string(),
        Some(root) => format!("{root}/{glob}"),
        None => glob.to_string(),
    }
}

fn glob_matches(glob: &str, files: &[PathBuf]) -> bool {
    let Ok(glob) = GlobBuilder::new(glob).literal_separator(false).build() else {
        return true;
    };
    let set = GlobSet::builder().add(glob).build();
    let Ok(set) = set else {
        return true;
    };
    files.iter().any(|path| set.is_match(path))
}
