use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::{Context, Result};
use rayon::prelude::*;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

mod domain;
use domain::{banned_domain_needles, email_domain, message, normalize_domain};

pub const RULE_ID: &str = "test-email-domain-policy";

const DEFAULT_EXTENSIONS: &[&str] = &[
    ".cjs", ".css", ".csv", ".cts", ".html", ".js", ".jsx", ".json", ".jsonc", ".md", ".mjs",
    ".mts", ".sql", ".ts", ".tsx", ".txt", ".xml", ".yml", ".yaml",
];

const EMAIL_PATTERN: &str = r"(?i)[a-z0-9._%+\$\{\}-]+@[a-z0-9.-]+\.[a-z]{2,}|[a-z0-9._%+\$\{\}-]+%40[a-z0-9.-]*(?:\.|%2e)[a-z0-9.%+-]+";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) banned_domains: Vec<String>,
    pub(crate) allowed_email_patterns: Vec<String>,
    pub(crate) replacement: Option<String>,
    pub(crate) extensions: Vec<String>,
}

struct CompiledOptions {
    email: Regex,
    banned_domains: HashSet<String>,
    domain_needles: Vec<Box<[u8]>>,
    allowed: Vec<Regex>,
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
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &target_roots))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan_with_sources(root, &opts, &files, sources)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan_with_sources(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    if opts.banned_domains.is_empty() {
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
        .map(|domain| normalize_domain(domain))
        .collect();
    let domain_needles = banned_domain_needles(&banned_domains);
    Ok(CompiledOptions {
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

fn check_file_with_sources(
    root: &Path,
    path: &Path,
    opts: &CompiledOptions,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    if !opts.extensions.iter().any(|ext| rel.ends_with(ext)) {
        return Vec::new();
    }
    let Some(content) = super::read_source(sources, path) else {
        return Vec::new();
    };
    if !opts.domain_needles.is_empty()
        && !super::byte_prefilter::contains_any_ascii_ignore_case(
            content.as_bytes(),
            &opts.domain_needles,
        )
    {
        return Vec::new();
    }
    scan_email_lines(&rel, &content, opts)
}

fn scan_email_lines(rel: &str, content: &str, opts: &CompiledOptions) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for (index, line) in content.lines().enumerate() {
        for matched in opts.email.find_iter(line).map(|m| m.as_str()) {
            if opts.allowed.iter().any(|allowed| allowed.is_match(matched)) {
                continue;
            }
            let domain = email_domain(matched);
            if !opts.banned_domains.contains(&domain) {
                continue;
            }
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: rel.to_string(),
                line: index + 1,
                message: message(rel, &domain, opts.replacement.as_deref()),
                import: Some(matched.to_string()),
                target: Some(domain),
            });
        }
    }
    findings
}

#[cfg(test)]
mod tests;
