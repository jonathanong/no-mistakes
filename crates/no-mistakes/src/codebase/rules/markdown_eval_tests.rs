use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use regex::Regex;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "markdown-eval-tests";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) allow: Vec<String>,
}

struct CompiledOptions {
    include: GlobMatcher,
    allow: BTreeSet<String>,
    markdown: Regex,
    shell: Regex,
    evals: Regex,
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
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.try_rule_options()?;
        let compiled = compile_options(&opts)?;
        findings.extend(super::allow_audit::missing_file_findings(
            root,
            config,
            rule,
            sources,
            "allow",
            opts.allow.iter().map(|file| {
                let path = crate::codebase::ts_resolver::normalize_path(&root.join(file));
                let rel = relative_slash_path(root, &path);
                (
                    if compiled.include.is_match(&rel) {
                        file.clone()
                    } else {
                        String::new()
                    },
                    file.clone(),
                )
            }),
        ));
        let files =
            super::selected_rule_files_with_sources(root, config, rule, all_files, sources)?;
        let files: Vec<PathBuf> = files
            .into_iter()
            .filter(|path| compiled.include.is_match(&relative_slash_path(root, path)))
            .collect();
        findings.extend(scan::scan(root, &compiled, &files, sources));
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    Ok(CompiledOptions {
        include: GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?,
        allow: opts.allow.iter().cloned().collect(),
        markdown: Regex::new(r#"['"`][^'"`]*\.md['"`]"#)?,
        shell: Regex::new(
            r#"\b(?:execFileSync|execSync|spawnSync|execFile|exec|spawn)\(\s*['"`](?:bash|sh|zsh|/bin/bash|/bin/sh)['"`]"#,
        )?,
        evals: Regex::new(r"\beval\b")?,
    })
}

fn is_eval_test(source: &str, opts: &CompiledOptions) -> bool {
    opts.markdown.is_match(source) && opts.shell.is_match(source) && opts.evals.is_match(source)
}

#[cfg(test)]
mod tests;
