use super::policy::{tokens, Token};

// Match the replacement around this occurrence, rather than exempting every
// occurrence of a denied token whenever the name contains one valid replacement.
pub(super) fn contains(name: &str, denied: &Token, replacement: &str) -> bool {
    tokens(replacement).iter().any(|part| {
        if !part.text.eq_ignore_ascii_case(&denied.text) {
            return false;
        }
        let Some(start) = denied.start.checked_sub(part.start) else {
            return false;
        };
        let end = start + replacement.len();
        name.get(start..end)
            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(replacement))
            && (start == 0 || name.as_bytes()[start - 1] == b'_')
            && (end == name.len() || name.as_bytes()[end] == b'_')
    })
}
