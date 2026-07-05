use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct PreserveRootOptions {
    roots: Option<Vec<PathBuf>>,
}

pub(super) fn filesystem_rule_target_roots(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    rule_ids: &[&str],
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for rule_id in rule_ids {
        roots.extend(filesystem_rule_preserved_roots(root, config, rule_id));
    }
    sort_dedup_roots(roots)
}

fn filesystem_rule_preserved_roots(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    rule_id: &str,
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for rule in config.rule_applications(rule_id) {
        if rule_id == super::FORBIDDEN_WORKSPACE_CLOSURE {
            roots.push(root.to_path_buf());
        }
        roots.extend(super::super::target_roots(root, config, rule));
        if !rule_supports_discovery_roots(rule_id) {
            continue;
        }
        let opts: PreserveRootOptions = rule.rule_options();
        if let Some(option_roots) = opts.roots {
            roots.extend(
                option_roots
                    .into_iter()
                    .map(|path| normalize_rule_root(root, path)),
            );
        }
    }
    sort_dedup_roots(roots)
}

fn rule_supports_discovery_roots(rule_id: &str) -> bool {
    matches!(
        rule_id,
        super::AGENTS_MD_MAX_SIZE
            | super::RUST_MAX_LINES_PER_FILE
            | super::RUST_NO_INLINE_ALLOWS
            | super::RUST_NO_INLINE_TESTS
    )
}

fn sort_dedup_roots(mut roots: Vec<PathBuf>) -> Vec<PathBuf> {
    roots.sort();
    roots.dedup();
    roots
}

fn normalize_rule_root(root: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

pub(super) fn filesystem_rule_files(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    rule_id: &str,
    files: &[PathBuf],
) -> Vec<PathBuf> {
    let preserved_roots = filesystem_rule_preserved_roots(root, config, rule_id);
    let skip = super::super::skip_dir_set(config);
    files
        .iter()
        .filter(|path| {
            super::super::file_allowed_by_roots_and_skip(root, &skip, path, &preserved_roots)
        })
        .cloned()
        .collect()
}
