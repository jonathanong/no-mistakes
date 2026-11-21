use super::output_syntax::unquote;

pub(super) fn same_name_expansion(value: &str, name: &str) -> bool {
    let value = unquote(value.trim().trim_end_matches(';').trim());
    if let Some(bare) = bare_ref(value) {
        return bare == name;
    }
    let Some(inner) = braced(value) else {
        return false;
    };
    inner == name
        || inner
            .strip_prefix(name)
            .is_some_and(|rest| removed(rest) || sliced(rest))
}

fn bare_ref(value: &str) -> Option<&str> {
    let rest = value.strip_prefix('$')?;
    (!rest.starts_with('{')).then_some(rest)
}

fn braced(value: &str) -> Option<&str> {
    value
        .strip_prefix("${")
        .and_then(|body| body.strip_suffix('}'))
}

fn removed(rest: &str) -> bool {
    let pattern = rest
        .strip_prefix("%%")
        .or_else(|| rest.strip_prefix("##"))
        .or_else(|| rest.strip_prefix('%'))
        .or_else(|| rest.strip_prefix('#'));
    pattern.is_some_and(|pattern| !pattern.is_empty() && !pattern.contains('$'))
}

fn sliced(rest: &str) -> bool {
    let Some(body) = rest.strip_prefix(':') else {
        return false;
    };
    let (offset, length) = body
        .split_once(':')
        .map(|(offset, length)| (offset, Some(length)))
        .unwrap_or((body, None));
    digits(offset) && length.is_none_or(digits)
}

fn digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}
