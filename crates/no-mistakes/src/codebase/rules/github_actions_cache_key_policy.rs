use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::Path;

mod broad_hash;
mod expand;
mod execution;
mod forbidden;
mod judge;
mod local_action;
mod options;
mod output;
mod output_syntax;
mod scan;
mod shell_cmd;
mod text;

pub const RULE_ID: &str = "github-actions-cache-key-policy";

#[doc(hidden)]
pub use execution::diagnostics_with_files_and_sources;
super::diagnostic_source_rule_entrypoints!();

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    super::check_sourced_standalone(&root, config, check_with_files)
}

pub(crate) fn collect_config_errors(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
    errors: &mut Vec<String>,
) {
    for (index, rule) in super::validate_options::indexed_rule_applications(config, RULE_ID) {
        let mut app_errors = std::collections::BTreeSet::new();
        if let Err(error) = super::path_filter::RulePathFilter::new(root, config, rule) {
            app_errors.insert(format!("{error:#}"));
        }
        match rule.try_rule_options::<options::Options>() {
            Ok(opts) => options::collect_errors(&opts, &mut app_errors),
            Err(error) => {
                app_errors.insert(format!("{error:#}"));
            }
        }
        super::validate_options::push_application_errors(
            root,
            config,
            index,
            config_path,
            app_errors,
            errors,
        );
    }
}

fn hit(file: &str, line: usize, message: String) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.into(),
        file: file.into(),
        line,
        message,
        import: None,
        target: None,
    }
}

#[cfg(test)]
mod tests;
