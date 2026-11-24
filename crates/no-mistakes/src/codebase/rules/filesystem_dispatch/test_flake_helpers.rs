pub(super) mod allow;
pub(super) mod ast;
pub(super) mod config_ast;

pub(super) use allow::{AllowEntry, CompiledAllowList};

use crate::codebase::rules::path_filter::{filter_rule_files, GlobMatcher};
use crate::codebase::rules::{self, RuleFinding};
use crate::codebase::ts_source::{byte_offset_to_line, relative_slash_path, SourceStore};
use crate::config::v2::schema::{RuleDef, StringOrList};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use oxc_span::Span;
use rayon::prelude::*;
use serde::de::DeserializeOwned;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const SCRIPT_EXTENSIONS: &[&str] = &[".cjs", ".cts", ".js", ".jsx", ".mjs", ".mts", ".ts", ".tsx"];

pub(super) fn compile_optional_globs(
    patterns: &[String],
    label: &str,
) -> Result<Option<GlobMatcher>> {
    if patterns.is_empty() {
        Ok(None)
    } else {
        Ok(Some(GlobMatcher::new(patterns, label)?))
    }
}

pub(super) fn configured_paths(configs: &Option<StringOrList>) -> BTreeSet<String> {
    configs
        .as_ref()
        .map(|value| {
            value
                .values()
                .into_iter()
                .map(|item| item.trim_start_matches("./").replace('\\', "/"))
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn is_script(rel: &str) -> bool {
    SCRIPT_EXTENSIONS.iter().any(|ext| rel.ends_with(ext))
}

pub(super) fn is_playwright_config_name(rel: &str) -> bool {
    let base = rel.rsplit('/').next().unwrap_or(rel);
    (base.starts_with("playwright.config.") || base.starts_with("playwright-ct.config."))
        && is_script(base)
}

pub(super) fn is_vitest_config_name(rel: &str) -> bool {
    let base = rel.rsplit('/').next().unwrap_or(rel);
    (base.starts_with("vitest.config.") || base.starts_with("vite.config.")) && is_script(base)
}

pub(super) struct RunContext<'a> {
    pub(super) rule_id: &'static str,
    pub(super) root: &'a Path,
    pub(super) config: &'a NoMistakesConfig,
    pub(super) all_files: &'a [PathBuf],
    pub(super) sources: &'a SourceStore,
    pub(super) defer_suppression: bool,
}

pub(super) fn run_rule_applications<O, C, F, G, S>(
    ctx: RunContext<'_>,
    compile: F,
    file_globs: G,
    scan: S,
) -> Result<Vec<RuleFinding>>
where
    O: DeserializeOwned + Default,
    C: Sync,
    F: Fn(O, &NoMistakesConfig) -> Result<C> + Sync,
    G: Fn(&C) -> Option<&GlobMatcher> + Sync,
    S: Fn(&Path, &Path, &C, &SourceStore) -> Vec<RuleFinding> + Sync,
{
    let per_rule: Result<Vec<Vec<RuleFinding>>> = ctx
        .config
        .rule_applications(ctx.rule_id)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let compiled = compile(rule.try_rule_options()?, ctx.config)?;
            let files = select_files(
                ctx.root,
                ctx.config,
                rule,
                ctx.all_files,
                file_globs(&compiled),
            )?;
            let mut findings: Vec<RuleFinding> = files
                .par_iter()
                .flat_map(|path| scan(ctx.root, path, &compiled, ctx.sources))
                .collect();
            if let Some(prefix) = rule.message.as_deref() {
                for finding in &mut findings {
                    finding.message = format!("{prefix} {}", finding.message);
                }
            }
            if !ctx.defer_suppression {
                rules::suppress_rule_findings_with_sources(ctx.root, &mut findings, ctx.sources);
            }
            Ok(findings)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = per_rule?.into_iter().flatten().collect();
    rules::sort_findings(&mut findings);
    Ok(findings)
}

fn select_files(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    all_files: &[PathBuf],
    file_globs: Option<&GlobMatcher>,
) -> Result<Vec<PathBuf>> {
    let target_roots = rules::target_roots(root, config, rule);
    let skip = rules::skip_dir_set(config);
    let scoped: Vec<PathBuf> = all_files
        .iter()
        .filter(|path| rules::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots))
        .cloned()
        .collect();
    let filtered = filter_rule_files(root, config, rule, &scoped)?;
    let Some(matcher) = file_globs else {
        return Ok(filtered);
    };
    Ok(filtered
        .into_iter()
        .filter(|path| matcher.is_match(&relative_slash_path(root, path)))
        .collect())
}

pub(super) fn make_finding(
    rule: &str,
    file: &str,
    source: &str,
    span: Span,
    message: String,
    target: Option<String>,
) -> RuleFinding {
    RuleFinding {
        rule: rule.to_string(),
        file: file.to_string(),
        line: byte_offset_to_line(source, span.start as usize) as usize,
        message,
        import: None,
        target,
        causes: Vec::new(),
    }
}
