use std::collections::BTreeMap;

pub(super) const PULL_REQUEST_READ: &str = "github.event.pull_request.";

/// What one end of a diff range names, as far as this rule can tell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum End {
    BaseSha,
    BaseRef,
    HeadSha,
    GithubSha,
    Other,
}

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

pub(super) fn classify(piece: &str, exprs: &[String], env: &BTreeMap<String, String>) -> End {
    let piece = piece.trim_matches(|ch| ch == '"' || ch == '\'');
    if let Some(name) = variable(piece) {
        let Some(value) = env.get(name) else {
            return End::Other;
        };
        let (text, inner) = extract(value.trim());
        return classify_literal(&text, &inner);
    }
    classify_literal(piece, exprs)
}

fn classify_literal(piece: &str, exprs: &[String]) -> End {
    let piece = piece
        .strip_prefix("refs/remotes/origin/")
        .or_else(|| piece.strip_prefix("origin/"))
        .unwrap_or(piece);
    let Some(expression) = piece
        .strip_prefix("__ghx")
        .and_then(|rest| rest.strip_suffix("__"))
        .and_then(|digits| digits.parse::<usize>().ok())
        .and_then(|index| exprs.get(index))
    else {
        return End::Other;
    };
    match expression.as_str() {
        "github.event.pull_request.base.sha" => End::BaseSha,
        "github.base_ref" => End::BaseRef,
        "github.event.pull_request.head.sha" => End::HeadSha,
        "github.sha" => End::GithubSha,
        expression => fallback_end(expression).unwrap_or(End::Other),
    }
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

fn fallback_end(expression: &str) -> Option<End> {
    let (left, right) = expression.split_once("||")?;
    match (left.trim(), right.trim()) {
        ("github.event.pull_request.base.sha", "github.event.merge_group.base_sha")
        | ("github.event.merge_group.base_sha", "github.event.pull_request.base.sha") => {
            Some(End::BaseSha)
        }
        ("github.event.pull_request.head.sha", "github.event.merge_group.head_sha")
        | ("github.event.merge_group.head_sha", "github.event.pull_request.head.sha") => {
            Some(End::HeadSha)
        }
        _ => None,
    }
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
                .filter(|_| fallback_end(expression).is_none())
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
