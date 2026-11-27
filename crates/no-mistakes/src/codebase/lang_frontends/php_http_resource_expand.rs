use regex::Regex;
use std::sync::OnceLock;

#[path = "php_http_resource_paths.rs"]
mod paths;

pub(super) use paths::{expand_actions, static_path};

pub(super) struct ActionFilters {
    pub only: Option<Vec<String>>,
    pub except: Option<Vec<String>>,
}

pub(super) const API_ACTIONS: &[&str] = &["index", "store", "show", "update", "destroy"];
const RESOURCE_ACTIONS: &[&str] = &[
    "index", "create", "store", "show", "edit", "update", "destroy",
];

pub(super) fn filters(options: &str, tail: &str) -> Option<ActionFilters> {
    let only = merged_list(options, tail, "only")?;
    let except = merged_list(options, tail, "except")?;
    if only.is_some() && except.is_some() {
        return None;
    }
    Some(ActionFilters { only, except })
}

fn merged_list(options: &str, tail: &str, key: &str) -> Option<Option<Vec<String>>> {
    let from_array = keyed_list(options, &format!("array-{key}"));
    let from_named = keyed_list(options, &format!("named-{key}"));
    let from_chain = keyed_list(tail, &format!("chain-{key}"));
    if mentions(options, key) && from_array.is_none() && from_named.is_none() {
        return None;
    }
    if tail.contains(&format!("->{key}")) && from_chain.is_none() {
        return None;
    }
    let mut chosen: Option<Vec<String>> = None;
    for list in [from_array, from_named, from_chain].into_iter().flatten() {
        if chosen.as_ref().is_some_and(|existing| existing != &list) {
            return None;
        }
        chosen.get_or_insert(list);
    }
    Some(chosen)
}

fn mentions(options: &str, key: &str) -> bool {
    options.contains(&format!("'{key}'"))
        || options.contains(&format!("\"{key}\""))
        || options.contains(&format!("{key}:"))
}

fn keyed_list(text: &str, kind: &str) -> Option<Vec<String>> {
    list_re(kind)?
        .captures(text)
        .and_then(|caps| parse_list(caps.get(1)?.as_str()))
}

fn list_re(kind: &str) -> Option<&'static Regex> {
    let (index, pattern) = match kind {
        "array-only" => (0, r#"['"]only['"]\s*=>\s*\[([^\]]*)\]"#),
        "array-except" => (1, r#"['"]except['"]\s*=>\s*\[([^\]]*)\]"#),
        "named-only" => (2, r"\bonly\s*:\s*\[([^\]]*)\]"),
        "named-except" => (3, r"\bexcept\s*:\s*\[([^\]]*)\]"),
        "chain-only" => (4, r"->only\s*\(\s*\[([^\]]*)\]\s*\)"),
        "chain-except" => (5, r"->except\s*\(\s*\[([^\]]*)\]\s*\)"),
        _ => return None,
    };
    static RE: [OnceLock<Regex>; 6] = [
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
        OnceLock::new(),
    ];
    Some(RE[index].get_or_init(|| Regex::new(pattern).expect("resource filter")))
}

fn parse_list(inner: &str) -> Option<Vec<String>> {
    let inner = inner.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    if inner.contains('$') {
        return None;
    }
    inner
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(quoted)
        .collect()
}

fn quoted(text: &str) -> Option<String> {
    let text = text.trim().trim_end_matches(',');
    let bytes = text.as_bytes();
    let quote = *bytes.first()?;
    if (quote != b'\'' && quote != b'"') || bytes.len() < 2 || *bytes.last()? != quote {
        return None;
    }
    let body = &text[1..text.len() - 1];
    (!body.contains('\\')).then(|| body.to_string())
}

pub(super) fn bare_form(path: &str, only: Option<&[String]>, except: Option<&[String]>) -> bool {
    !path.contains('.') && only.is_none() && except.is_none()
}

pub(super) fn selected<'a>(
    api: bool,
    only: Option<&[String]>,
    except: Option<&[String]>,
) -> Option<Vec<&'a str>> {
    let base: &[&str] = if api { API_ACTIONS } else { RESOURCE_ACTIONS };
    let known = |name: &String| base.contains(&name.as_str());
    if let Some(only) = only {
        if !only.iter().all(known) {
            return None;
        }
        return Some(
            base.iter()
                .copied()
                .filter(|action| only.iter().any(|name| name == action))
                .collect(),
        );
    }
    if let Some(except) = except {
        if !except.iter().all(known) {
            return None;
        }
        return Some(
            base.iter()
                .copied()
                .filter(|action| except.iter().all(|name| name != action))
                .collect(),
        );
    }
    Some(base.to_vec())
}
