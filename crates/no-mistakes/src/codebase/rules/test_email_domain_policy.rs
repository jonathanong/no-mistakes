use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::{Context, Result};
use rayon::prelude::*;
use regex::Regex;
use serde::Deserialize;
use std::cell::Cell;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

thread_local! {
    pub(super) static PREFILTER_SKIPS: Cell<u32> = const { Cell::new(0) };
}

mod domain;
mod encoding;
mod scan;
use super::allow_audit::ExceptionAudit;
use domain::{banned_domain_needles, email_domain, message, normalize_domain};
use scan::check_file_with_sources;
#[cfg(test)]
use scan::scan_email_lines;

pub const RULE_ID: &str = "test-email-domain-policy";

const DEFAULT_EXTENSIONS: &[&str] = &[
    ".cjs", ".css", ".csv", ".cts", ".html", ".js", ".jsx", ".json", ".jsonc", ".md", ".mjs",
    ".mts", ".sql", ".ts", ".tsx", ".txt", ".xml", ".yml", ".yaml",
];

const EMAIL_PATTERN: &str = r"(?i)[a-z0-9._%+\$\{\}-]+@[a-z0-9.-]+\.[a-z]{2,}|[a-z0-9._%+\$\{\}-]+%40[a-z0-9.-]*(?:\.|%2e)[a-z0-9.%+-]+";

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) banned_domains: Option<Vec<String>>,
    #[serde(default)]
    pub(crate) allowed_email_patterns: Vec<String>,
    #[serde(default)]
    pub(crate) replacement: Option<String>,
    #[serde(default)]
    pub(crate) extensions: Vec<String>,
}

struct CompiledOptions {
    email: Regex,
    banned_domains: HashSet<String>,
    domain_needles: Vec<Box<[u8]>>,
    allowed: Vec<Regex>,
    audit: Option<ExceptionAudit>,
    extensions: Vec<String>,
    replacement: Option<String>,
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
            let opts: Options = rule.try_rule_options()?;
            anyhow::ensure!(
                opts.banned_domains.is_some(),
                "{RULE_ID}: bannedDomains is required"
            );
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &target_roots))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            if opts.banned_domains.as_ref().is_none_or(Vec::is_empty) {
                return Ok(Vec::new());
            }
            let mut compiled = compile_options(&opts)?;
            compiled.audit = Some(ExceptionAudit::new(
                root,
                config,
                rule,
                sources,
                "allowedEmailPatterns",
                &opts.allowed_email_patterns,
            ));
            let mut findings: Vec<_> = files
                .par_iter()
                .flat_map(|path| check_file_with_sources(root, path, &compiled, sources))
                .collect();
            findings.extend(compiled.audit.as_ref().unwrap().finish(sources, |path| {
                let rel = relative_slash_path(root, path);
                compiled.extensions.iter().any(|ext| rel.ends_with(ext))
            }));
            Ok(findings)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

#[cfg(test)]
fn scan_with_sources(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    if opts.banned_domains.as_ref().is_none_or(Vec::is_empty) {
        return Ok(Vec::new());
    }
    let compiled = compile_options(opts)?;
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| check_file_with_sources(root, path, &compiled, sources))
        .collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    let extensions = if opts.extensions.is_empty() {
        DEFAULT_EXTENSIONS
            .iter()
            .map(|ext| ext.to_string())
            .collect()
    } else {
        opts.extensions.clone()
    };
    let banned_domains: HashSet<String> = opts
        .banned_domains
        .iter()
        .flatten()
        .map(|domain| normalize_domain(domain))
        .collect();
    let domain_needles = banned_domain_needles(&banned_domains);
    Ok(CompiledOptions {
        audit: None,
        email: Regex::new(EMAIL_PATTERN).expect("email regex is valid"),
        banned_domains,
        domain_needles,
        allowed: opts
            .allowed_email_patterns
            .iter()
            .map(|pattern| {
                Regex::new(pattern).with_context(|| {
                    format!("{RULE_ID} contains invalid allowed pattern `{pattern}`")
                })
            })
            .collect::<Result<Vec<_>>>()?,
        extensions,
        replacement: opts.replacement.clone(),
    })
}

#[cfg(test)]
mod tests;
