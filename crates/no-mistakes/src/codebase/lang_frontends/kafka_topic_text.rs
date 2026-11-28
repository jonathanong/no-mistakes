pub(super) fn complete_topic(source: &str, start: usize, end: usize, text: &str) -> Option<String> {
    let quote = source[..start].chars().next_back()?;
    if (quote != '"' && quote != '\'')
        || text.contains('\n')
        || text.contains('\r')
        || concatenated(source, end)
    {
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
    reject_dollar: bool,
) -> Option<String> {
    let quote = source[..start].chars().next_back()?;
    let text = complete_topic(source, start, end, text)?;
    if quote == '"' && text.contains("#{") {
        return None;
    }
    if reject_dollar && dollar_template(quote, &text) {
        return None;
    }
    Some(text)
}

pub(super) fn quoted_element(element: &str, reject_dollar: bool) -> Option<String> {
    let quote = element.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let inner = element.strip_prefix(quote)?.strip_suffix(quote)?;
    if inner.is_empty()
        || inner.contains(quote)
        || inner.contains('\n')
        || inner.contains('\r')
        || (reject_dollar && dollar_template(quote, inner))
    {
        return None;
    }
    Some(inner.to_string())
}

pub(super) fn bare_subscribe_literal(element: &str, reject_dollar: bool) -> Option<String> {
    let quote = element.chars().next()?;
    let inner = quoted_element(element, reject_dollar)?;
    if quote == '"' && inner.contains("#{") {
        return None;
    }
    if inner.starts_with('^') || inner.contains('*') {
        return None;
    }
    Some(inner)
}

fn dollar_template(quote: char, text: &str) -> bool {
    if quote != '"' {
        return false;
    }
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
    let Some(quote) = rest.chars().next() else {
        return false;
    };
    rest[quote.len_utf8()..].trim_start().starts_with('+')
}
