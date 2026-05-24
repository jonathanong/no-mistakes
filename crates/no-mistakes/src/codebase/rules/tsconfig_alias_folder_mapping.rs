use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "tsconfig-alias-folder-mapping";

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct AliasMapping {
    pub(crate) prefix: String,
    pub(crate) root: String,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) tsconfig: PathBuf,
    pub(crate) base_dir: String,
    pub(crate) mappings: Vec<AliasMapping>,
    /// When true, also verify that each expected target directory exists on disk.
    pub(crate) check_exists: bool,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        if opts.tsconfig.as_os_str().is_empty() || opts.mappings.is_empty() {
            continue;
        }
        findings.extend(check_tsconfig(root, &opts)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

/// Check using a pre-discovered file list — tsconfig is read directly regardless.
pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    _files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    check(root, config)
}

fn check_tsconfig(root: &Path, opts: &Options) -> Result<Vec<RuleFinding>> {
    let tsconfig_path = root.join(&opts.tsconfig);
    let ts_config = match crate::codebase::ts_resolver::load_tsconfig(&tsconfig_path) {
        Ok(c) => c,
        Err(_) => return Ok(Vec::new()),
    };
    if ts_config.paths.is_empty() {
        return Ok(Vec::new());
    }
    let tsconfig_str = relative_slash_path(root, &tsconfig_path);
    let all_paths: BTreeMap<String, Vec<String>> = ts_config.paths.into_iter().collect();
    let mut findings = Vec::new();
    for (alias, targets) in &all_paths {
        findings.extend(check_alias(
            root,
            &tsconfig_str,
            alias,
            targets,
            opts,
            &all_paths,
        ));
    }
    Ok(findings)
}

fn check_alias(
    root: &Path,
    tsconfig_str: &str,
    alias: &str,
    targets: &[String],
    opts: &Options,
    all_paths: &BTreeMap<String, Vec<String>>,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();

    // Check alias → target direction
    for mapping in &opts.mappings {
        let alias_prefix = format!("{}/", mapping.prefix);
        if alias.starts_with(&alias_prefix) {
            let suffix = &alias[alias_prefix.len()..];
            let expected = format!("{}/{}/{}", opts.base_dir, mapping.root, suffix);
            for target in targets {
                if target != &expected {
                    findings.push(make_finding(
                        tsconfig_str,
                        &format!("{tsconfig_str}: {alias} must target {expected}, not {target}"),
                    ));
                } else if opts.check_exists {
                    let dir_path = expected.trim_end_matches("/*").trim_end_matches('*');
                    let abs_dir = root.join(dir_path.trim_start_matches("./"));
                    if !abs_dir.is_dir() {
                        findings.push(make_finding(
                            tsconfig_str,
                            &format!("{tsconfig_str}: {alias} targets {dir_path}, which does not exist on disk"),
                        ));
                    }
                }
            }
        }
    }

    // Check target → alias direction
    for mapping in &opts.mappings {
        let target_prefix = format!("{}/{}/", opts.base_dir, mapping.root);
        let required_alias_prefix = format!("{}/", mapping.prefix);
        for target in targets {
            if target.starts_with(&target_prefix) {
                // This target belongs to this mapping; the alias must use the right prefix
                if !alias.starts_with(&required_alias_prefix) {
                    // Avoid double-reporting: only flag if not already caught from a different mapping
                    let already_flagged = findings.iter().any(|f: &RuleFinding| {
                        f.message.contains(alias) && f.message.contains(target)
                    });
                    if !already_flagged {
                        // Check the alias isn't valid for any mapping by looking at all known targets
                        let all_targets =
                            all_paths.get(alias).map(Vec::as_slice).unwrap_or(targets);
                        let valid = opts.mappings.iter().any(|m| {
                            let ap = format!("{}/", m.prefix);
                            if alias.starts_with(&ap) {
                                let suffix = &alias[ap.len()..];
                                let expected = format!("{}/{}/{}", opts.base_dir, m.root, suffix);
                                all_targets.contains(&expected)
                            } else {
                                false
                            }
                        });
                        if !valid {
                            findings.push(make_finding(
                                tsconfig_str,
                                &format!(
                                    "{tsconfig_str}: {alias} must use prefix {}, not target {target} directly",
                                    mapping.prefix
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }

    findings
}

fn make_finding(file: &str, message: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message: message.to_string(),
        import: None,
        target: None,
    }
}

#[cfg(test)]
mod tests;
