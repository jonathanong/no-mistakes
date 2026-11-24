use std::collections::BTreeMap;

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
    while let Some(start) = rest.find("${{") {
        let Some(len) = rest[start + 3..].find("}}") else {
            break;
        };
        text.push_str(&rest[..start]);
        text.push_str(&format!("__ghx{}__", exprs.len()));
        exprs.push(rest[start + 3..start + 3 + len].trim().to_string());
        rest = &rest[start + 3 + len + 2..];
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
    let Some(index) = piece
        .strip_prefix("__ghx")
        .and_then(|rest| rest.strip_suffix("__"))
        .and_then(|digits| digits.parse::<usize>().ok())
    else {
        return End::Other;
    };
    match exprs.get(index).map(String::as_str) {
        Some("github.event.pull_request.base.sha") => End::BaseSha,
        Some("github.base_ref") => End::BaseRef,
        Some("github.event.pull_request.head.sha") => End::HeadSha,
        Some("github.sha") => End::GithubSha,
        _ => End::Other,
    }
}

fn variable(piece: &str) -> Option<&str> {
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
