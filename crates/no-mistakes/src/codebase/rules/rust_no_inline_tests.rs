use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::Path;

pub const RULE_ID: &str = "rust-no-inline-tests";

mod ast;

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    super::rust_rules_combined::standalone::check(root, config, RULE_ID)
}

pub(crate) fn findings_from_parsed(
    path: &Path,
    root: &Path,
    parsed: &syn::File,
) -> Vec<RuleFinding> {
    let file = relative_slash_path(root, path);
    ast::cfg_test_lines(parsed)
        .into_iter()
        .map(|line| RuleFinding {
            rule: RULE_ID.to_string(),
            file: file.clone(),
            line,
            message: "inline #[cfg(test)] item - move test-only code to a sibling test module and keep only #[cfg(test)] mod tests; in source".to_string(),
            import: None,
            target: None,
        })
        .collect()
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_support;
