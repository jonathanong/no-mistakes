use super::{options::Options, selectors};
use crate::config::source_locations::ConfigPath;
use crate::config::v2::schema::{OptionLocation, RuleDef};
use std::path::Path;

pub(in crate::codebase::rules) fn dependencies(rule: &RuleDef, root: &Path) -> anyhow::Result<()> {
    let options = rule.strict_rule_options::<Options>()?;
    let path = ConfigPath::default().key("options");
    let selector = |raw: &str| selectors::validate(root, raw);
    validate_globs(rule, path.clone().key("roots"), &options.roots, selector)?;
    validate_globs(rule, path.clone().key("excludeRoots"), &options.exclude_roots, selector)?;
    validate_globs(rule, path.clone().key("forbiddenModules"), &options.forbidden_modules, glob_syntax)?;
    validate_globs(rule, path.clone().key("forbiddenFiles"), &options.forbidden_files, glob_syntax)?;
    for (index, exception) in options.exceptions.iter().enumerate() {
        let entry = path.clone().key("exceptions").index(index);
        validate_globs(rule, entry.clone().key("roots"), &exception.roots, selector)?;
        validate_globs(rule, entry.key("allowedFiles"), &exception.allowed_files, glob_syntax)?;
    }
    Ok(())
}

pub(in crate::codebase::rules) fn workspace_closure(rule: &RuleDef) -> anyhow::Result<()> {
    let options = rule.strict_rule_options::<super::super::forbidden_workspace_closure::options::Options>()?;
    validate_globs(
        rule,
        ConfigPath::default().key("options").key("forbidden"),
        &options.forbidden,
        glob_syntax,
    )
}

fn glob_syntax(raw: &str) -> Result<(), globset::Error> {
    globset::Glob::new(raw).map(drop)
}

fn validate_globs(
    rule: &RuleDef,
    path: ConfigPath,
    patterns: &[String],
    validate: impl Fn(&str) -> Result<(), globset::Error>,
) -> anyhow::Result<()> {
    for (index, pattern) in patterns.iter().enumerate() {
        validate(pattern).map_err(|error| {
            OptionLocation::invalid(
                rule,
                path.clone().index(index),
                format_args!("invalid glob: {error}"),
            )
        })?;
    }
    Ok(())
}
