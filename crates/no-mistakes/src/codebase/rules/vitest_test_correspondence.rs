mod helpers;
pub(crate) use helpers::source_candidates;
use helpers::{check_source_to_test, stem_and_dir};

use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "vitest-test-correspondence";

const DEFAULT_TEST_EXTENSIONS: &[&str] = &[".test.mts", ".test.ts", ".test.tsx"];
const DEFAULT_TESTS_DIR: &str = "__tests__";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) scopes: Vec<String>,
    pub(crate) test_extensions: Vec<String>,
    pub(crate) tests_dir: String,
    pub(crate) direction: Direction,
}

#[derive(Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Direction {
    #[default]
    Both,
    TestToSource,
    SourceToTest,
}

impl Direction {
    fn checks_test_to_source(self) -> bool {
        matches!(self, Self::Both | Self::TestToSource)
    }

    fn checks_source_to_test(self) -> bool {
        matches!(self, Self::Both | Self::SourceToTest)
    }
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let skip = &config.filesystem.skip_directories;
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.rule_options();
            let target_roots = super::target_roots(root, config, rule);
            let files: Vec<PathBuf> = target_roots
                .iter()
                .flat_map(|r| discover_files(r, skip))
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan(root, &opts, &files)
        })
        .collect();
    merge(all)
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
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| target_roots.iter().any(|r| p.starts_with(r)))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan(root, &opts, &files)
        })
        .collect();
    merge(all)
}

fn merge(all: Result<Vec<Vec<RuleFinding>>>) -> Result<Vec<RuleFinding>> {
    let mut v: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut v);
    Ok(v)
}

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let exts: Vec<&str> = if opts.test_extensions.is_empty() {
        DEFAULT_TEST_EXTENSIONS.to_vec()
    } else {
        opts.test_extensions.iter().map(String::as_str).collect()
    };
    let tdir: &str = if opts.tests_dir.is_empty() {
        DEFAULT_TESTS_DIR
    } else {
        &opts.tests_dir
    };
    let sep = format!("/{tdir}/");
    let pre = format!("{tdir}/");

    let rel_set: HashSet<String> = files.iter().map(|p| relative_slash_path(root, p)).collect();

    let test_files: Vec<(String, &str)> = files
        .iter()
        .filter_map(|p| {
            let rel = relative_slash_path(root, p);
            if !opts.scopes.is_empty()
                && !opts
                    .scopes
                    .iter()
                    .any(|s| rel == *s || rel.starts_with(&format!("{s}/")))
            {
                return None;
            }
            exts.iter().find(|&&e| rel.ends_with(e)).map(|&e| (rel, e))
        })
        .collect();

    let mut findings = Vec::new();
    let mut dir_stems: HashMap<String, Vec<String>> = HashMap::new();

    if opts.direction.checks_test_to_source() {
        for (rel, test_ext) in &test_files {
            if rel.contains(sep.as_str()) || rel.starts_with(pre.as_str()) {
                continue; // in __tests__ — exempt
            }
            let (dir, base) = stem_and_dir(rel, test_ext);
            dir_stems.entry(dir.clone()).or_default().push(base.clone());
            let found = source_candidates(&dir, &base, test_ext)
                .iter()
                .any(|c| rel_set.contains(c.as_str()));
            if !found {
                findings.push(RuleFinding {
                    rule: RULE_ID.to_string(),
                    file: rel.clone(),
                    line: 1,
                    message: format!("{rel}: no corresponding source file found"),
                    import: None,
                    target: None,
                });
            }
        }

        if opts.direction == Direction::Both {
            // Duplicate stem detection
            for (dir, stems) in &dir_stems {
                let mut seen = HashSet::new();
                let dups: HashSet<&str> = stems
                    .iter()
                    .filter(|s| !seen.insert(s.as_str()))
                    .map(String::as_str)
                    .collect();
                for dup in dups {
                    for (rel, test_ext) in &test_files {
                        let (fdir, fbase) = stem_and_dir(rel, test_ext);
                        if !rel.contains(sep.as_str())
                            && !rel.starts_with(pre.as_str())
                            && fdir == *dir
                            && fbase == dup
                        {
                            findings.push(RuleFinding {
                                rule: RULE_ID.to_string(),
                                file: rel.clone(),
                                line: 1,
                                message: format!(
                                    "{rel}: duplicate-stem test files must live under {tdir}/"
                                ),
                                import: None,
                                target: None,
                            });
                        }
                    }
                }
            }
        }
    }

    if opts.direction.checks_source_to_test() {
        findings.extend(check_source_to_test(
            files, root, opts, &exts, &rel_set, tdir,
        ));
    }
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}

#[cfg(test)]
mod tests;
