use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "package-json-registry-only";

mod lockfile;

const BLOCKED_PREFIXES: &[&str] = &[
    "git:",
    "git+ssh:",
    "git+https:",
    "github:",
    "bitbucket:",
    "gitlab:",
    "gist:",
    "http:",
    "https:",
    "file:",
    "link:",
    "portal:",
    "patch:",
];

#[rustfmt::skip]
const DEP_FIELDS: &[&str] = &["dependencies", "devDependencies", "peerDependencies", "optionalDependencies"];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) lockfile: Option<PathBuf>,
    pub(crate) scopes: Vec<PathBuf>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let files = discover_files(root, &config.filesystem.skip_directories);
    check_with_files(root, config, &files)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.rule_options();
            let target_roots = super::target_roots(root, config, rule);
            let rule_filter = super::path_filter::RulePathFilter::new(root, config, rule)?;
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &target_roots))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            Ok(scan(
                root,
                &opts,
                &files,
                &target_roots,
                &rule_filter,
                sources,
            ))
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

pub(crate) fn is_blocked_specifier(spec: &str) -> bool {
    if spec.starts_with("workspace:") || spec.starts_with("catalog:") {
        return false;
    }
    if let Some(rest) = spec.strip_prefix("npm:") {
        if let Some(after_at) = rest.strip_prefix('@') {
            let version = after_at.find('@').map_or("", |i| &after_at[i + 1..]);
            return !version.is_empty() && is_blocked_specifier(version);
        }
        return is_blocked_specifier(rest.rfind('@').map_or(rest, |i| &rest[i + 1..]));
    }
    BLOCKED_PREFIXES.iter().any(|p| spec.starts_with(p))
        || (!spec.starts_with('@') && spec.contains('/'))
        || (spec.starts_with('@') && spec.splitn(3, '/').count() > 2)
}

fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    target_roots: &[PathBuf],
    rule_filter: &super::path_filter::RulePathFilter,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n == "package.json")
                && !p.components().any(|c| c.as_os_str() == "node_modules")
                && (opts.scopes.is_empty()
                    || opts.scopes.iter().any(|s| {
                        p.starts_with(if s.is_absolute() {
                            s.clone()
                        } else {
                            root.join(s)
                        })
                    }))
        })
        .flat_map(|path| check_package_json_with_sources(path, root, sources))
        .collect();
    findings.extend(
        target_roots
            .par_iter()
            .flat_map(|lockfile_root| {
                let Some(lockfile_path) = &opts.lockfile else {
                    return Vec::new();
                };
                if !rule_filter.is_match(&lockfile_root.join(lockfile_path)) {
                    return Vec::new();
                }
                lockfile::check(root, lockfile_root, opts, sources)
            })
            .collect::<Vec<_>>(),
    );
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    findings
}

fn check_package_json_with_sources(
    path: &Path,
    root: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let Ok(json) = sources.parse_json_path(path) else {
        return Vec::new();
    };
    check_package_json_value(path, root, &json)
}

fn check_package_json_value(
    path: &Path,
    root: &Path,
    json: &serde_json::Value,
) -> Vec<RuleFinding> {
    let file = relative_slash_path(root, path);
    let mut findings = Vec::new();
    for field in DEP_FIELDS {
        let Some(deps) = json.get(field).and_then(|v| v.as_object()) else {
            continue;
        };
        let mut sorted: Vec<_> = deps.iter().collect();
        sorted.sort_by_key(|(k, _)| k.as_str());
        for (dep, val) in sorted {
            let spec = val.as_str().unwrap_or("");
            if is_blocked_specifier(spec) {
                findings.push(RuleFinding {
                    rule: RULE_ID.to_string(),
                    file: file.clone(),
                    line: 1,
                    message: format!(
                        "{file}: \"{dep}\": \"{spec}\" is not allowed \
                        (only npm registry / workspace: / catalog: / npm: aliases permitted)"
                    ),
                    import: None,
                    target: None,
                });
            }
        }
    }
    findings
}

#[cfg(test)]
mod tests;
