use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
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
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let skip = &config.filesystem.skip_directories;
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        let target_roots = super::target_roots(root, config, rule);
        let files: Vec<PathBuf> = target_roots
            .iter()
            .flat_map(|r| discover_files(r, skip))
            .collect();
        findings.extend(scan(root, &opts, &files)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        let target_roots = super::target_roots(root, config, rule);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|p| target_roots.iter().any(|r| p.starts_with(r)))
            .cloned()
            .collect();
        findings.extend(scan(root, &opts, &files)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn test_extensions(opts: &Options) -> Vec<&str> {
    if opts.test_extensions.is_empty() {
        DEFAULT_TEST_EXTENSIONS.to_vec()
    } else {
        opts.test_extensions.iter().map(String::as_str).collect()
    }
}

fn tests_dir(opts: &Options) -> &str {
    if opts.tests_dir.is_empty() {
        DEFAULT_TESTS_DIR
    } else {
        &opts.tests_dir
    }
}

fn stem_and_dir(rel: &str, test_ext: &str) -> (String, String) {
    let stem = rel.strip_suffix(test_ext).unwrap_or(rel);
    match stem.rfind('/') {
        Some(i) => (stem[..i].to_string(), stem[i + 1..].to_string()),
        None => (String::new(), stem.to_string()),
    }
}

pub(crate) fn source_candidates(dir: &str, stem: &str, test_ext: &str) -> Vec<String> {
    let p = if dir.is_empty() {
        String::new()
    } else {
        format!("{dir}/")
    };
    let src_ext = test_ext.rsplit('.').next().unwrap_or("ts");
    match src_ext {
        "mts" | "cts" => vec![
            format!("{p}{stem}.{src_ext}"),
            format!("{p}index.{src_ext}"),
        ],
        "js" | "jsx" => vec![
            format!("{p}{stem}.js"),
            format!("{p}{stem}.jsx"),
            format!("{p}index.js"),
            format!("{p}index.jsx"),
        ],
        _ => vec![
            format!("{p}{stem}.ts"),
            format!("{p}{stem}.tsx"),
            format!("{p}index.ts"),
            format!("{p}index.tsx"),
        ],
    }
}

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let exts = test_extensions(opts);
    let tdir = tests_dir(opts);
    let sep = format!("/{tdir}/");
    let pre = format!("{tdir}/");

    let rel_set: HashSet<String> = files.iter().map(|p| relative_slash_path(root, p)).collect();

    let test_files: Vec<(String, &str)> = files
        .iter()
        .filter_map(|p| {
            let rel = relative_slash_path(root, p);
            if !opts.scopes.is_empty() && !opts.scopes.iter().any(|s| rel.starts_with(s.as_str())) {
                return None;
            }
            exts.iter().find(|&&e| rel.ends_with(e)).map(|&e| (rel, e))
        })
        .collect();

    let mut findings = Vec::new();
    let mut dir_stems: HashMap<String, Vec<String>> = HashMap::new();

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

    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}

#[cfg(test)]
mod tests;
