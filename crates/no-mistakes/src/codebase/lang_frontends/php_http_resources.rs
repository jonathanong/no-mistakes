use regex::Regex;
use std::sync::OnceLock;

#[path = "php_http_resource_expand.rs"]
mod expand;
#[path = "php_http_resource_scan.rs"]
mod scan;

pub(super) fn extract_filtered_resources(source: &str) -> Vec<(String, String)> {
    let mut routes = Vec::new();
    for caps in resource_call_re().captures_iter(source) {
        let Some(kind) = caps.get(1).map(|item| item.as_str()) else {
            continue;
        };
        let Some(args_at) = caps.get(0).map(|item| item.end()) else {
            continue;
        };
        routes.extend(expand_call(source, kind, args_at).into_iter().flatten());
    }
    routes
}

fn expand_call(source: &str, kind: &str, args_at: usize) -> Option<Vec<(String, String)>> {
    let close = scan::matching_paren(source, args_at)?;
    let args = scan::split_args(&source[args_at..close])?;
    let tail = scan::statement_tail(source, close + 1);
    if kind == "apiResources" {
        return expand_api_resources(&args, tail);
    }
    let call = positional_or_named(&args, kind == "apiResource")?;
    if !expand::static_path(&call.path) {
        return None;
    }
    let filters = expand::filters(&call.options, tail)?;
    if expand::bare_form(
        &call.path,
        filters.only.as_deref(),
        filters.except.as_deref(),
    ) {
        return None;
    }
    let actions = expand::selected(call.api, filters.only.as_deref(), filters.except.as_deref())?;
    Some(expand::expand_actions(&call.path, &call.handler, &actions))
}

struct Call {
    api: bool,
    path: String,
    handler: String,
    options: String,
}

fn positional_or_named(args: &[&str], api: bool) -> Option<Call> {
    if args
        .iter()
        .any(|arg| arg.contains("name:") || arg.contains("controller:"))
    {
        return Some(Call {
            api,
            path: named_value(args, "name").and_then(quoted)?,
            handler: named_value(args, "controller").and_then(class_name)?,
            options: options_arg(args),
        });
    }
    Some(Call {
        api,
        path: quoted(args.first()?)?,
        handler: class_name(args.get(1)?)?,
        options: args.get(2).copied().unwrap_or("").to_string(),
    })
}

fn options_arg(args: &[&str]) -> String {
    args.iter()
        .find(|arg| {
            let trimmed = arg.trim();
            trimmed.starts_with('[')
                || trimmed.starts_with("options:")
                || trimmed.starts_with("only:")
                || trimmed.starts_with("except:")
        })
        .copied()
        .unwrap_or("")
        .to_string()
}

fn expand_api_resources(args: &[&str], tail: &str) -> Option<Vec<(String, String)>> {
    if args.len() != 1 || tail.contains("->only") || tail.contains("->except") {
        return None;
    }
    let body = args[0].trim().strip_prefix('[')?.strip_suffix(']')?;
    if body.contains('$') {
        return None;
    }
    let mut routes = Vec::new();
    for caps in map_pair_re().captures_iter(body) {
        let path = caps.get(1)?.as_str();
        if !expand::static_path(path) {
            return None;
        }
        let handler = class_name(caps.get(2)?.as_str())?;
        routes.extend(expand::expand_actions(path, &handler, expand::API_ACTIONS));
    }
    (!routes.is_empty()).then_some(routes)
}

fn named_value<'a>(args: &'a [&'a str], key: &str) -> Option<&'a str> {
    args.iter().find_map(|arg| {
        let (name, value) = arg.trim().split_once(':')?;
        (name.trim() == key).then_some(value.trim())
    })
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

fn class_name(text: &str) -> Option<String> {
    let class = text
        .trim()
        .trim_end_matches(',')
        .strip_suffix("::class")?
        .trim();
    if class.is_empty()
        || !class
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '\\')
    {
        return None;
    }
    Some(class.replace('\\', ".").trim_start_matches('.').to_string())
}

fn resource_call_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"Route::(apiResources|apiResource|resource)\s*\(").expect("resource call")
    })
}

fn map_pair_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"['"]([^'"]+)['"]\s*=>\s*((?:\\?[A-Za-z_][A-Za-z0-9_\\]*)::class)"#)
            .expect("api resources pair")
    })
}
