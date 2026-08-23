use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "csharp-no-async-void-delegate";

pub(super) const DEFAULT_MESSAGE: &str =
    "do not pass an async lambda to a void Action API; wrap as () => _ = FooAsync()";

const DEFAULT_CONSTRUCTORS: &[&str] = &["Command"];
const DEFAULT_METHODS: &[&str] = &["BeginInvokeOnMainThread"];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) constructors: Vec<String>,
    pub(crate) methods: Vec<String>,
    pub(crate) allow: Vec<String>,
    pub(crate) message: Option<String>,
}

pub(super) struct CompiledOptions {
    pub(super) constructors: Vec<regex::Regex>,
    pub(super) methods: Vec<regex::Regex>,
    pub(super) allow: GlobMatcher,
    pub(super) message: String,
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
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    check_with_files_sources_and_deferred_suppression(root, config, all_files, sources, false)
}

pub(crate) fn check_with_files_sources_and_deferred_suppression(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts = compile_options(rule.rule_options()?)?;
        let target_roots = super::target_roots(root, config, rule);
        let skip = super::skip_dir_set(config);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| {
                is_csharp_file(path)
                    && super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots)
            })
            .cloned()
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        findings.extend(scan::scan(root, &opts, &files, sources, defer_suppression));
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_options(opts: Options) -> Result<CompiledOptions> {
    let constructors = if opts.constructors.is_empty() {
        constructor_regexes(DEFAULT_CONSTRUCTORS.iter().copied())
    } else {
        constructor_regexes(opts.constructors.iter().map(String::as_str))
    };
    let methods = if opts.methods.is_empty() {
        method_regexes(DEFAULT_METHODS.iter().copied())
    } else {
        method_regexes(opts.methods.iter().map(String::as_str))
    };
    Ok(CompiledOptions {
        constructors,
        methods,
        allow: GlobMatcher::new(&opts.allow, &format!("{RULE_ID} allow"))?,
        message: match opts.message.filter(|hint| !hint.is_empty()) {
            Some(hint) => format!("{DEFAULT_MESSAGE} {hint}"),
            None => DEFAULT_MESSAGE.to_string(),
        },
    })
}

fn constructor_regexes<'a>(names: impl Iterator<Item = &'a str>) -> Vec<regex::Regex> {
    names
        .filter(|name| !name.is_empty())
        .map(|name| {
            let escaped = regex::escape(name);
            regex::Regex::new(&format!(
                r"new\s+(?:[\w.:]+\.)?\b{escaped}(?:<[^>]+>)?\s*\(\s*async\b"
            ))
            .expect("constructor pattern")
        })
        .collect()
}

fn method_regexes<'a>(names: impl Iterator<Item = &'a str>) -> Vec<regex::Regex> {
    names
        .filter(|name| !name.is_empty())
        .map(|name| {
            let escaped = regex::escape(name);
            regex::Regex::new(&format!(r"(?:[\w.:]+\.)?\b{escaped}\s*\(\s*async\b"))
                .expect("method pattern")
        })
        .collect()
}

fn is_csharp_file(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("cs")
}

#[cfg(test)]
mod tests;
