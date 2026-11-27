use super::revisions::classify_fallback;

pub(super) const PULL_REQUEST_READ: &str = "github.event.pull_request.";

/// Replace each `${{ expr }}` with a whitespace-free placeholder so a shell
/// word split cannot cut an expression apart. Returns the new text and the
/// trimmed expressions in placeholder order.
pub(super) fn extract(script: &str) -> (String, Vec<String>) {
    let mut text = String::new();
    let mut exprs = Vec::new();
    let mut rest = script;
    while let Some((prefix, expression, suffix)) = split_expression(rest) {
        text.push_str(prefix);
        text.push_str(&format!("__ghx{}__", exprs.len()));
        exprs.push(expression.trim().to_string());
        rest = suffix;
    }
    text.push_str(rest);
    (text, exprs)
}

pub(super) fn variable(piece: &str) -> Option<&str> {
    let name = piece
        .strip_prefix("${")
        .and_then(|rest| rest.strip_suffix('}'))
        .or_else(|| piece.strip_prefix('$'))?;
    let mut chars = name.chars();
    let first = chars.next()?;
    (first == '_' || first.is_ascii_alphabetic())
        .then_some(name)
        .filter(|_| chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric()))
}

fn split_expression(text: &str) -> Option<(&str, &str, &str)> {
    let (prefix, tail) = text.split_once("${{")?;
    let (expression, suffix) = tail.split_once("}}")?;
    Some((prefix, expression, suffix))
}

pub(super) fn unguarded_pull_request_read(text: &str) -> Option<usize> {
    let mut rest = text;
    let mut offset = 0;
    while let Some((prefix, expression, suffix)) = split_expression(rest) {
        let read = prefix.find(PULL_REQUEST_READ).or_else(|| {
            expression
                .find(PULL_REQUEST_READ)
                .filter(|_| classify_fallback(expression).is_none())
                .map(|read| prefix.len() + 3 + read)
        });
        if let Some(read) = read {
            return Some(offset + read);
        }
        offset += rest.len() - suffix.len();
        rest = suffix;
    }
    rest.find(PULL_REQUEST_READ).map(|read| offset + read)
}
