use std::collections::BTreeMap;

pub(super) fn singularize(token: &str, overrides: &BTreeMap<String, String>) -> String {
    if let Some(value) = overrides
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(token))
        .map(|(_, value)| value.clone())
    {
        return value;
    }
    if let Some(stem) = cut(token, "ies") {
        return format!("{stem}y");
    }
    if cut(token, "sses").is_some() {
        return token[..token.len() - 2].to_string();
    }
    if cut(token, "xes").is_some() || cut(token, "ches").is_some() || cut(token, "shes").is_some() {
        return token[..token.len() - 2].to_string();
    }
    if let Some(stem) = cut(token, "s") {
        return stem.to_string();
    }
    token.to_string()
}

pub(super) fn singular_name(name: &str, overrides: &BTreeMap<String, String>) -> String {
    match name.rsplit_once('_') {
        Some((prefix, last)) if !last.is_empty() => {
            format!("{prefix}_{}", singularize(last, overrides))
        }
        _ => singularize(name, overrides),
    }
}

pub(super) fn last_token(name: &str) -> &str {
    name.rsplit('_')
        .find(|part| !part.is_empty())
        .unwrap_or(name)
}

fn cut<'a>(token: &'a str, suffix: &str) -> Option<&'a str> {
    token
        .len()
        .checked_sub(suffix.len())
        .filter(|start| token.is_char_boundary(*start))
        .filter(|_| token[token.len() - suffix.len()..].eq_ignore_ascii_case(suffix))
        .map(|start| &token[..start])
}

pub(super) fn join_or(items: &[String]) -> String {
    items.join(" or ")
}
