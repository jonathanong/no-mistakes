use globset::GlobBuilder;

/// Ast-grep compiles string ignores with globset's defaults (`literal_separator`
/// false) and `{glob, caseInsensitive}` objects with that flag. `*` matches a
/// leading dot. The pattern is matched as written against the path relative to
/// the `sgconfig` directory. A leading `./` on the pattern is not rewritten:
/// scan strips `./` from the file path before matching.
pub(super) fn matches_tracked(
    pattern: &str,
    case_insensitive: bool,
    project: &str,
    tracked: &[String],
) -> Result<bool, ()> {
    let matcher = GlobBuilder::new(pattern)
        .case_insensitive(case_insensitive)
        .literal_separator(false)
        .build()
        .map_err(|_| ())?
        .compile_matcher();
    Ok(tracked.iter().any(|relative| {
        project_relative(relative, project).is_some_and(|candidate| matcher.is_match(candidate))
    }))
}

fn project_relative<'a>(relative: &'a str, project: &str) -> Option<&'a str> {
    if project.is_empty() {
        return Some(relative);
    }
    relative.strip_prefix(project)?.strip_prefix('/')
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "globs_tests.rs"]
mod tests;
