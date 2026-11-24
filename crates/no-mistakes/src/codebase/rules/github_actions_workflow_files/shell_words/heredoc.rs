use crate::codebase::rules::github_actions_download_integrity::words::parse;
use std::borrow::Cow;

pub(super) fn mask(script: &str) -> Option<Cow<'_, str>> {
    if !script.contains("<<") {
        return Some(Cow::Borrowed(script));
    }
    let parsed = parse(script);
    if parsed.unterminated_heredoc {
        return None;
    }
    let ranges = parsed.heredoc_ranges;
    if ranges.is_empty() {
        return Some(Cow::Borrowed(script));
    }
    let mut chars: Vec<_> = script.chars().collect();
    // The scanner tracks one pending delimiter, so two openers on the same
    // command line cannot be assigned separate bodies reliably.
    if ranges
        .windows(2)
        .any(|pair| !chars[pair[0].start..pair[1].start].contains(&'\n'))
    {
        return None;
    }
    for range in ranges {
        for ch in &mut chars[range] {
            if *ch != '\n' {
                *ch = ' ';
            }
        }
    }
    Some(Cow::Owned(chars.into_iter().collect()))
}
