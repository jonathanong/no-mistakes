use crate::integration_tests::project_config::prefix_globs;
use crate::playwright::test_patterns::Patterns;
use std::path::Path;

pub(super) fn materialize_patterns(
    root: &Path,
    test_dir: &Path,
    patterns: &Patterns,
    visible: &crate::fx::PathSet,
    include: bool,
) -> Vec<String> {
    let globs = if include && !patterns.warnings.is_empty() {
        vec!["**/*".to_string()]
    } else {
        patterns.globs.clone()
    };
    let mut result = prefix_globs(root, test_dir, &globs);
    if patterns.regexes.is_empty() {
        return result;
    }
    let mut matched = visible
        .iter()
        .filter(|path| path.starts_with(test_dir) && patterns.regex_matches(path))
        .map(|path| globset::escape(&crate::codebase::ts_source::relative_slash_path(root, path)))
        .collect::<Vec<_>>();
    matched.sort();
    result.extend(matched);
    result
}
