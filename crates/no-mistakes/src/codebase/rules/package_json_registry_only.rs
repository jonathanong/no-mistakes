use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "package-json-registry-only";

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

const DEP_FIELDS: &[&str] = &[
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
];

const BLOCKED_RESOLUTION_KEYS: &[&str] = &["tarball", "repo", "commit", "directory"];

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
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.rule_options();
            let target_roots = super::target_roots(root, config, rule);
            let rule_filter = super::path_filter::RulePathFilter::new(root, config, rule)?;
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| target_roots.iter().any(|r| p.starts_with(r)))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            Ok(scan(root, &opts, &files, &target_roots, &rule_filter))
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
        .flat_map(|path| check_package_json(path, root))
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
                check_lockfile(root, lockfile_root, opts)
            })
            .collect::<Vec<_>>(),
    );
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    findings
}

fn check_package_json(path: &Path, root: &Path) -> Vec<RuleFinding> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
        return Vec::new();
    };
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

fn check_lockfile(root: &Path, lockfile_root: &Path, opts: &Options) -> Vec<RuleFinding> {
    let Some(lockfile_path) = &opts.lockfile else {
        return Vec::new();
    };
    let lockfile_abs = lockfile_root.join(lockfile_path);
    let Ok(content) = std::fs::read_to_string(&lockfile_abs) else {
        return Vec::new();
    };
    let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) else {
        return Vec::new();
    };
    let file = relative_slash_path(root, &lockfile_abs);
    let Some(packages) = yaml.get("packages").and_then(|p| p.as_mapping()) else {
        return Vec::new();
    };
    let mut pairs: Vec<(&serde_yaml::Value, &serde_yaml::Value)> = packages.iter().collect();
    pairs.sort_by(|(a, _), (b, _)| a.as_str().unwrap_or("").cmp(b.as_str().unwrap_or("")));
    let mut findings = Vec::new();
    for (key, pkg_val) in pairs {
        let pkg_name = key.as_str().unwrap_or("");
        let Some(resolution) = pkg_val.get("resolution") else {
            continue;
        };
        for &blocked_key in BLOCKED_RESOLUTION_KEYS {
            if resolution.get(blocked_key).is_some() {
                findings.push(RuleFinding {
                    rule: RULE_ID.to_string(),
                    file: file.clone(),
                    line: 1,
                    message: format!(
                        "{file}: package \"{pkg_name}\" has a non-registry \
                        resolution ({blocked_key}) \u{2014} only npm registry packages are permitted"
                    ),
                    import: None,
                    target: None,
                });
                break;
            }
        }
    }
    findings
}

#[cfg(test)]
mod tests;
