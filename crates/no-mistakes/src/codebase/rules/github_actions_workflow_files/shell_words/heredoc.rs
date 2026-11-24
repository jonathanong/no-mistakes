use crate::codebase::rules::github_actions_download_integrity::words::parse;
use std::borrow::Cow;

pub(super) fn mask(script: &str) -> (Cow<'_, str>, bool) {
    if !script.contains("<<") {
        return (Cow::Borrowed(script), false);
    }
    let parsed = parse(script);
    let ranges = parsed.heredoc_ranges;
    if ranges.is_empty() {
        return (Cow::Borrowed(script), parsed.unterminated_heredoc);
    }
    let mut chars: Vec<_> = script.chars().collect();
    for range in ranges {
        for ch in &mut chars[range] {
            if *ch != '\n' {
                *ch = ' ';
            }
        }
    }
    (Cow::Owned(chars.into_iter().collect()), parsed.unterminated_heredoc)
}
