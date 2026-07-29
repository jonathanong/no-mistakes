use super::char_scan::{is_access_boundary, previous_non_whitespace, quoted_end, static_access};

/// Extract every `needs.<x>` / `steps.<x>` access (dot or bracket form)
/// directly in `condition`, sorted and deduplicated. `context` is `"needs"`
/// or `"steps"` and is matched case-sensitively (unlike the output-chain
/// scanner, which matches `needs` case-insensitively).
pub fn static_references(condition: Option<&str>, context: &str) -> Vec<String> {
    static_references_with_case(condition, context, true)
}

pub(super) fn static_references_case_insensitive(
    condition: Option<&str>,
    context: &str,
) -> Vec<String> {
    static_references_with_case(condition, context, false)
}

fn static_references_with_case(
    condition: Option<&str>,
    context: &str,
    case_sensitive: bool,
) -> Vec<String> {
    let Some(condition) = condition else {
        return Vec::new();
    };
    let chars: Vec<char> = condition.chars().collect();
    let context_chars: Vec<char> = context.chars().collect();
    let mut references = std::collections::BTreeMap::new();
    let mut index = 0usize;
    while index < chars.len() {
        let character = chars[index];
        if character == '\'' || character == '"' {
            index = quoted_end(&chars, index, character);
            continue;
        }
        let candidate = chars.get(index..index + context_chars.len());
        let starts_with_context = candidate.is_some_and(|candidate| {
            if case_sensitive {
                candidate == context_chars
            } else {
                candidate
                    .iter()
                    .zip(&context_chars)
                    .all(|(actual, expected)| actual.eq_ignore_ascii_case(expected))
            }
        });
        if !starts_with_context
            || !is_access_boundary(previous_non_whitespace(&chars, index as isize - 1))
        {
            index += 1;
            continue;
        }
        let access_start = index + context_chars.len();
        match static_access(&chars, access_start) {
            Some((reference, end)) => {
                let sort_key = if case_sensitive {
                    reference.clone()
                } else {
                    reference.to_ascii_lowercase()
                };
                references.entry(sort_key).or_insert(reference);
                index = end;
            }
            None => index = access_start,
        }
    }
    references.into_values().collect()
}
