use super::RuleFinding;
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::ts_source::{
    has_disable_comment, has_disable_file_comment, relative_slash_path,
};
use crate::config::v2::schema::NoMistakesConfig;
use anyhow::{bail, Result};
pub(crate) use ast::extract_program;
use rayon::prelude::*;
use serde::Serialize;
use std::path::{Path, PathBuf};

mod ast;
mod patterns;
mod visitor;

pub const RULE_ID: &str = "nextjs-no-caching";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NextjsCachingFinding {
    pub(crate) line: usize,
    pub(crate) message: String,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let files =
        crate::codebase::ts_source::discover_files(&root, &config.filesystem.skip_directories);
    let files: Vec<_> = files
        .into_iter()
        .filter(|path| is_indexable(path))
        .collect();
    check_files(&root, config, &files)
}

pub(crate) fn check_with_facts(
    root: &Path,
    config: &NoMistakesConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let target_roots = super::target_roots(&root, config, rule);
        for path in shared.files() {
            let Some(facts) = shared.ts.get(path) else {
                continue;
            };
            if !target_roots
                .iter()
                .any(|target_root| path.starts_with(target_root))
            {
                continue;
            }
            let Some(source) = facts.source.as_ref() else {
                bail!("{} requires source facts for {}", RULE_ID, path.display());
            };
            let Some(cache_facts) = facts.nextjs_caching.as_ref() else {
                bail!(
                    "{} requires Next.js caching facts for {}",
                    RULE_ID,
                    path.display()
                );
            };
            findings.extend(findings_for_file(&root, path, source, cache_facts));
        }
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn check_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let target_roots = super::target_roots(root, config, rule);
        let rule_findings: Vec<RuleFinding> = files
            .par_iter()
            .filter(|path| {
                target_roots
                    .iter()
                    .any(|target_root| path.starts_with(target_root))
            })
            .flat_map(|path| {
                let Ok(source) = std::fs::read_to_string(path) else {
                    return Vec::new();
                };
                let Ok(cache_facts) = extract(path, &source) else {
                    return Vec::new();
                };
                findings_for_file(root, path, &source, &cache_facts)
            })
            .collect();
        findings.extend(rule_findings);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn findings_for_file(
    root: &Path,
    path: &Path,
    source: &str,
    cache_facts: &[NextjsCachingFinding],
) -> Vec<RuleFinding> {
    if has_disable_file_comment(source, RULE_ID) {
        return Vec::new();
    }
    let file = relative_slash_path(root, path);
    cache_facts
        .iter()
        .filter(|finding| !has_disable_comment(source, finding.line as u32, RULE_ID))
        .map(|finding| RuleFinding {
            rule: RULE_ID.to_string(),
            file: file.clone(),
            line: finding.line,
            message: finding.message.clone(),
            import: None,
            target: None,
        })
        .collect()
}

pub(crate) fn extract(path: &Path, source: &str) -> Result<Vec<NextjsCachingFinding>> {
    crate::ast::with_program(path, source, |program, _| extract_program(source, program))
}

#[cfg(test)]
mod tests;
