use super::manifest_analysis::ClassifiedFinding;
use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

mod included;
mod keys;
mod unavailable_includes;
use keys::{json_key_line, json_nested_key_line, yaml_has_key, yaml_top_level_key_line};

pub const RULE_ID: &str = "pnpm-overrides-ban";

const FIX: &str = "Fix dependency metadata upstream or upgrade the parent dependency instead.";

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let files = discover_files(root, &config.filesystem.skip_directories);
    check_with_files(root, config, &files)
}

super::borrowed_source_rule_entrypoints!();
mod effort;
pub(crate) use effort::check_with_files_sources_and_deferred_suppression;
pub use effort::diagnostics_with_files_and_sources;

mod scan;

#[cfg(test)]
mod tests;
