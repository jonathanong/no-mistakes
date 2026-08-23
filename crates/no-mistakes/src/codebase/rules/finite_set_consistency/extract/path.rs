use super::*;
use crate::config::v2::schema::RuleDef;
use crate::config::v2::NoMistakesConfig;
use std::collections::HashSet;

pub(in super::super) const PATH_REGEX_CAPTURE: &str = "path-regex-capture";

pub(in super::super) fn path_regex_capture_files(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    sources: &crate::codebase::ts_source::SourceStore,
    skip: &HashSet<&str>,
    target_roots: &[PathBuf],
    files: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    let mut extras = Vec::new();
    for path in sources.inventory().non_file_path_entry_paths() {
        if crate::codebase::rules::file_allowed_by_roots_and_skip(root, skip, &path, target_roots) {
            extras.push(path);
        }
    }
    let extras =
        crate::codebase::rules::path_filter::filter_rule_files(root, config, rule, &extras)?;
    let mut entries = files.to_vec();
    entries.extend(extras);
    entries.sort();
    entries.dedup();
    Ok(entries)
}

pub(in super::super) fn extract_path_regex_set(
    root: &Path,
    spec: &SetSpec,
    files: &[PathBuf],
    target_roots: &[PathBuf],
) -> Result<ExtractedSet> {
    let regex = Regex::new(&spec.pattern)?;
    let mut values = BTreeSet::new();
    for file in files {
        for rel in relative_paths_for_matching(root, file, target_roots) {
            let Some(captures) = regex.captures(&rel) else {
                continue;
            };
            let capture = match captures.name("value") {
                Some(capture) => Some(capture),
                None => captures.get(1),
            };
            if let Some(capture) = capture {
                values.insert(capture.as_str().to_string());
            }
        }
    }
    Ok(ExtractedSet {
        file: match spec.file.is_empty() {
            true => ".".to_string(),
            false => spec.file.clone(),
        },
        values,
        issues: Vec::new(),
    })
}

fn relative_paths_for_matching(root: &Path, file: &Path, target_roots: &[PathBuf]) -> Vec<String> {
    let mut paths = Vec::new();
    for target_root in target_roots {
        if file.starts_with(target_root) {
            paths.push(relative_slash_path(target_root, file));
        }
    }
    let repo_rel = relative_slash_path(root, file);
    if !paths.contains(&repo_rel) {
        paths.push(repo_rel);
    }
    paths
}
