use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Result;
use rayon::prelude::*;
use regex::Regex;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod analyze;
mod expressions;
mod hit;
mod line_scan;
mod value_walk;
mod yaml_parse;
mod yaml_text;

pub(crate) use line_scan::{step_uses_lines, StepUse};

pub const RULE_ID: &str = "github-actions-script-expressions";

/// Full-match patterns for constant contexts. `None` for `constantContexts`
/// selects these; an explicit list, including `[]`, replaces them.
pub(crate) const DEFAULT_CONSTANT_CONTEXTS: &[&str] = &[
    r"^github\.(repository|repository_owner|server_url|run_id|run_number|run_attempt|sha|event_name|workspace)$",
    r"^runner\.(os|arch|temp)$",
    r#"^job\.services\.[A-Za-z0-9_-]+\.ports\[(?:'[0-9]+'|"[0-9]+")\]$"#,
];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    /// `None` (option omitted) uses [`DEFAULT_CONSTANT_CONTEXTS`]. `Some` replaces
    /// that list, so `[]` flags every expression.
    pub(crate) constant_contexts: Option<Vec<String>>,
}

super::sourced_file_rule_entrypoints!();

fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let patterns = compile_patterns(opts)?;
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| check_file(path, root, &patterns, sources))
        .collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_patterns(opts: &Options) -> Result<Vec<Regex>> {
    let configured = opts
        .constant_contexts
        .clone()
        .unwrap_or_else(default_constant_contexts);
    configured
        .into_iter()
        .map(|pattern| {
            Regex::new(&pattern).map_err(|err| {
                anyhow::anyhow!(
                    "{RULE_ID} constantContexts pattern `{pattern}` is not a valid regex: {err}"
                )
            })
        })
        .collect()
}

pub(crate) fn default_constant_contexts() -> Vec<String> {
    DEFAULT_CONSTANT_CONTEXTS
        .iter()
        .map(|pattern| (*pattern).to_string())
        .collect()
}

fn check_file(
    path: &Path,
    root: &Path,
    patterns: &[Regex],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    if analyze::gha_kind(&rel).is_none() {
        return Vec::new();
    }
    let Some(content) = super::read_source(sources, path) else {
        return Vec::new();
    };
    analyze::check_source(&rel, &content, patterns)
}

#[cfg(test)]
mod tests;
