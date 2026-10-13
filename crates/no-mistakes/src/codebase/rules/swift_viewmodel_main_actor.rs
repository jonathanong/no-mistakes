use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "swift-viewmodel-main-actor";

pub(super) const DEFAULT_MESSAGE: &str =
    "ViewModel classes must be @MainActor so UI state is not mutated off the main actor";
pub(super) const DEFAULT_SUFFIX: &str = "ViewModel";
pub(super) const DEFAULT_ATTRIBUTE: &str = "MainActor";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) suffix: Option<String>,
    pub(crate) attribute: Option<String>,
    pub(crate) allow: Vec<String>,
    pub(crate) message: Option<String>,
}

pub(super) struct CompiledOptions {
    pub(super) suffix: String,
    pub(super) attribute: String,
    pub(super) allow: GlobMatcher,
    pub(super) message: String,
}

super::borrowed_source_rule_entrypoints!();

pub(crate) fn check_with_files_sources_and_deferred_suppression(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts = compile_options(rule.try_rule_options()?)?;
        let files =
            super::selected_rule_files_matching(root, config, rule, all_files, is_swift_file)?;
        findings.extend(scan::scan(root, &opts, &files, sources, defer_suppression));
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_options(opts: Options) -> Result<CompiledOptions> {
    Ok(CompiledOptions {
        suffix: match opts.suffix.filter(|s| !s.is_empty()) {
            Some(suffix) => suffix,
            None => DEFAULT_SUFFIX.to_string(),
        },
        attribute: match opts.attribute.filter(|s| !s.is_empty()) {
            Some(attribute) => attribute,
            None => DEFAULT_ATTRIBUTE.to_string(),
        },
        allow: GlobMatcher::new(&opts.allow, &format!("{RULE_ID} allow"))?,
        message: match opts.message.filter(|hint| !hint.is_empty()) {
            Some(hint) => format!("{DEFAULT_MESSAGE} {hint}"),
            None => DEFAULT_MESSAGE.to_string(),
        },
    })
}

fn is_swift_file(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("swift")
}

#[cfg(test)]
mod tests;
