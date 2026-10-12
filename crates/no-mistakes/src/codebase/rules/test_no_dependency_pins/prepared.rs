use super::*;
use crate::codebase::check_facts::CheckFactPlan;
use std::sync::Arc;

pub(crate) fn is_jsx_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("tsx" | "jsx")
    )
}

/// Declare only the rule-selected JSX files that need multiline assertion masking.
#[doc(hidden)]
pub fn configure_fact_plan(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    plan: &mut CheckFactPlan,
) -> Result<()> {
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.try_rule_options()?;
        let roots = super::super::target_roots(root, config, rule);
        let skip = super::super::skip_dir_set(config);
        let candidates: Vec<_> = files
            .iter()
            .filter(|path| super::super::file_allowed_by_roots_and_skip(root, &skip, path, &roots))
            .cloned()
            .collect();
        // Keep filtering errors ahead of pattern errors, as in standalone checks.
        let candidates =
            super::super::path_filter::filter_rule_files(root, config, rule, &candidates)?;
        let include = GlobMatcher::new(&opts.include, &format!("{RULE_ID} include"))?;
        // Custom patterns use line scanning. Compile regexes once during projection;
        // declaration only needs the static default multiline demand and include scope.
        if opts.patterns.is_empty() && DEFAULT_PATTERNS.iter().any(|(_, _, multiline)| *multiline) {
            let default_include = default_include_regex();
            Arc::make_mut(&mut plan.jsx_text_paths).extend(candidates.into_iter().filter(|path| {
                is_jsx_file(path) && {
                    let rel = relative_slash_path(root, path);
                    if include.is_empty() {
                        default_include.is_match(&rel)
                    } else {
                        include.is_match(&rel)
                    }
                }
            }));
        }
    }
    Ok(())
}

pub(crate) fn prepared_jsx_text_ranges(
    plan: &CheckFactPlan,
    path: &Path,
    program: &oxc_ast::ast::Program<'_>,
) -> Option<Vec<(usize, usize)>> {
    plan.jsx_text_paths
        .contains(path)
        .then(|| scan::jsx_text_ranges::collect_from_program(program))
}
