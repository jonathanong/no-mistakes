use super::types::SelectorMatcher;
pub use super::types::{compile_selector_regexes, compile_selector_regexes_with_html_ids};
pub(super) fn matcher_for_operator(operator: &str, value: &str) -> SelectorMatcher {
    match operator {
        "^=" => SelectorMatcher::Prefix(value.to_string()),
        "$=" => SelectorMatcher::Suffix(value.to_string()),
        "*=" => SelectorMatcher::Contains(value.to_string()),
        _ => SelectorMatcher::Exact(value.to_string()),
    }
}

pub(super) fn first_capture<'a>(
    captures: &'a regex::Captures<'_>,
    indexes: &[usize],
) -> Option<&'a str> {
    indexes
        .iter()
        .find_map(|index| captures.get(*index).map(|capture| capture.as_str()))
}
