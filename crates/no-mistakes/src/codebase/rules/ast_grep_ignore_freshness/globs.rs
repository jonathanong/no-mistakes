use globset::GlobBuilder;

/// Ast-grep compiles string ignores with globset's defaults (`literal_separator`
/// false) and `{glob, caseInsensitive}` objects with that flag. `*` matches a
/// leading dot. Try the tracked relative path and its `./` form so both spellings
/// of one inventory entry satisfy a pattern.
pub(super) fn matches_tracked(
    pattern: &str,
    case_insensitive: bool,
    tracked: &[String],
) -> Result<bool, ()> {
    let matcher = GlobBuilder::new(pattern)
        .case_insensitive(case_insensitive)
        .literal_separator(false)
        .build()
        .map_err(|_| ())?
        .compile_matcher();
    Ok(tracked
        .iter()
        .any(|relative| matcher.is_match(relative) || matcher.is_match(format!("./{relative}"))))
}
