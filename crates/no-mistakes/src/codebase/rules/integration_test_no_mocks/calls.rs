use regex::Regex;

use super::{byte_offset_to_line, RuleFinding, RULE_ID};

pub(super) fn findings(
    rel: &str,
    comments_removed: &str,
    calls: &[(String, Regex)],
) -> Vec<RuleFinding> {
    calls
        .iter()
        .flat_map(|(label, regex)| {
            regex
                .captures_iter(comments_removed)
                .filter_map(|captures| captures.name("call"))
                .filter(|matched| {
                    !super::strings::is_inside_string(comments_removed.as_bytes(), matched.start())
                })
                .filter(|matched| !line_starts_with_star(comments_removed, matched.start()))
                .map(|matched| RuleFinding {
                    rule: RULE_ID.to_string(),
                    file: rel.to_string(),
                    line: byte_offset_to_line(comments_removed, matched.start()) as usize,
                    message: format!(
                        "{rel}: integration tests must not use mocking libraries (`{label}`); use real dependencies and test helpers instead"
                    ),
                    import: Some(label.clone()),
                    target: None,
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn line_starts_with_star(source: &str, offset: usize) -> bool {
    let line_start = source[..offset]
        .rfind('\n')
        .map_or(0, |newline| newline + 1);
    source[line_start..offset].trim_start().starts_with('*')
}
