use super::Options;
use crate::codebase::rules::path_filter::GlobMatcher;
use crate::codebase::rules::validate_options::format_invalid_option;
use crate::codebase::rules::vitest_default_excludes::RULE_ID;
use std::collections::BTreeSet;

pub(super) fn collect_option_errors(options: &Options, errors: &mut BTreeSet<String>) {
    if let Some(config_path) = options
        .config
        .as_deref()
        .filter(|path| path.trim().is_empty())
    {
        errors.insert(invalid(
            "config",
            config_path,
            "an explicit non-empty Vitest config file path",
        ));
    }
    if options.config.is_none() && options.files.is_empty() {
        errors.insert(invalid(
            "files",
            &options.files,
            "a non-empty list of globs when options.config is absent",
        ));
    }
    for glob in &options.files {
        if glob.trim().is_empty() {
            errors.insert(invalid("files", glob, "a valid non-empty glob"));
        } else if let Err(error) = GlobMatcher::new(std::slice::from_ref(glob), RULE_ID) {
            errors.insert(invalid(
                "files",
                glob,
                format_args!("a valid non-empty glob (invalid glob: {})", error.root_cause()),
            ));
        }
    }
    for glob in options
        .required_globs
        .iter()
        .filter(|glob| glob.trim().is_empty())
    {
        errors.insert(invalid("requiredGlobs", glob, "a non-empty glob string"));
    }
}

pub(super) fn invalid(
    option: &str,
    value: impl std::fmt::Debug,
    allowed: impl std::fmt::Display,
) -> String {
    format_invalid_option(RULE_ID, option, value, allowed)
}
