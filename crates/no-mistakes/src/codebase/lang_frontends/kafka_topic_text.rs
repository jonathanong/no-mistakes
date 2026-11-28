#[derive(Clone, Copy)]
pub(super) enum DollarScope {
    None,
    DoubleQuote,
    AnyQuote,
}

pub(super) fn complete_topic(source: &str, start: usize, end: usize, text: &str) -> Option<String> {
    let quote = source[..start].chars().next_back()?;
    if concatenated(source, end) {
        return None;
    }
    let closing = source[end..].chars().next()?;
    (closing == quote).then(|| text.to_string())
}

pub(super) fn closed_literal(
    source: &str,
    start: usize,
    end: usize,
    text: &str,
    dollar_scope: DollarScope,
) -> Option<String> {
    let quote = source[..start].chars().next_back()?;
    let text = complete_topic(source, start, end, text)?;
    if quote == '"' && ruby_interpolation(&text) {
        return None;
    }
    if rejects_dollar(dollar_scope, quote, &text) {
        return None;
    }
    Some(text)
}

pub(super) fn quoted_element(element: &str, dollar_scope: DollarScope) -> Option<String> {
    let quote = element.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let inner = element.strip_prefix(quote)?.strip_suffix(quote)?;
    if inner.is_empty()
        || inner.contains(quote)
        || inner.contains(['\n', '\r'])
        || rejects_dollar(dollar_scope, quote, inner)
    {
        return None;
    }
    Some(inner.to_string())
}

pub(super) fn bare_subscribe_literal(element: &str, dollar_scope: DollarScope) -> Option<String> {
    let quote = element.chars().next()?;
    let inner = quoted_element(element, dollar_scope)?;
    if quote == '"' && ruby_interpolation(&inner) {
        return None;
    }
    if inner.starts_with('^') || inner.contains('*') {
        return None;
    }
    Some(inner)
}

fn ruby_interpolation(text: &str) -> bool {
    text.as_bytes()
        .windows(2)
        .any(|pair| pair[0] == b'#' && matches!(pair[1], b'{' | b'$' | b'@'))
}

fn rejects_dollar(scope: DollarScope, quote: char, text: &str) -> bool {
    let active = match scope {
        DollarScope::None => false,
        DollarScope::DoubleQuote => quote == '"',
        DollarScope::AnyQuote => true,
    };
    active && dollar_template(text)
}

fn dollar_template(text: &str) -> bool {
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            chars.next();
            continue;
        }
        if ch == '$' {
            match chars.peek() {
                Some('{') => return true,
                Some(next) if next.is_ascii_alphabetic() || *next == '_' => return true,
                _ => {}
            }
        }
    }
    false
}

fn concatenated(source: &str, end: usize) -> bool {
    let rest = source[end..].trim_start();
    rest.chars()
        .next()
        .is_some_and(|quote| rest[quote.len_utf8()..].trim_start().starts_with('+'))
}
