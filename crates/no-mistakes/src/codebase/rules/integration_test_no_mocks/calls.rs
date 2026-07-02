use regex::Regex;

use super::{byte_offset_to_line, RuleFinding, RULE_ID};

pub(super) fn findings(
    rel: &str,
    strings_removed: &str,
    calls: &[(String, Regex)],
) -> Vec<RuleFinding> {
    calls
        .iter()
        .flat_map(|(label, regex)| {
            regex
                .find_iter(strings_removed)
                .filter(|matched| !line_starts_with_star(strings_removed, matched.start()))
                .map(|matched| RuleFinding {
                    rule: RULE_ID.to_string(),
                    file: rel.to_string(),
                    line: byte_offset_to_line(strings_removed, matched.start()) as usize,
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
