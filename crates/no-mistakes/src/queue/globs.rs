use crate::queue::extract::FileFacts;
use crate::queue::extract_model::LocalCheck;
use globset::{GlobBuilder, GlobSet};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(super) fn record_empty_globs(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    facts: &mut HashMap<PathBuf, FileFacts>,
) {
    let relative = files
        .iter()
        .filter_map(|path| path.strip_prefix(root).ok().map(|path| path.to_path_buf()))
        .collect::<Vec<_>>();
    for project in config.projects.values() {
        for glob in prefixed(&project.root, &project.queues.enqueues)
            .into_iter()
            .chain(prefixed(&project.root, &project.queues.workers))
        {
            if glob_matches(&glob, &relative) {
                continue;
            }
            facts
                .entry(root.join(".no-mistakes.yml"))
                .or_default()
                .checks
                .push(LocalCheck {
                    line: 1,
                    kind: "empty-glob".to_string(),
                    job: None,
                    message: format!("queue glob '{glob}' matches no files"),
                });
        }
    }
}

fn prefixed(root: &Option<String>, globs: &[String]) -> Vec<String> {
    let prefix = root
        .as_deref()
        .map(str::trim)
        .map(|root| root.trim_start_matches("./").trim_end_matches('/'))
        .filter(|root| !root.is_empty() && *root != ".");
    globs
        .iter()
        .map(|glob| {
            let glob = glob.trim_start_matches("./");
            match prefix {
                Some(root) if glob == root || glob.starts_with(&format!("{root}/")) => {
                    glob.to_string()
                }
                Some(root) => format!("{root}/{glob}"),
                None => glob.to_string(),
            }
        })
        .collect()
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
