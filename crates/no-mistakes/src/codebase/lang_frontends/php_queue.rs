use super::{extract_named, extract_php_uses, php_namespace_re};
use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

pub(super) fn extract_php_requires(source: &str) -> Vec<String> {
    extract_named(source, php_require_re())
        .into_iter()
        .filter_map(|raw| {
            Path::new(&raw)
                .file_stem()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .collect()
}

fn php_require_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?:require|include)(?:_once)?\s*(?:\(?\s*(?:__DIR__\s*\.\s*)?)['"]([^'"]+)['"]"#,
        )
        .expect("php require")
    })
}

pub(super) fn extract_laravel_dispatches(source: &str) -> Vec<String> {
    let uses = extract_php_uses(source);
    let namespace = php_namespace_re()
        .captures(source)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().replace('\\', "."));
    let mut names = Vec::new();
    for raw in extract_named(source, laravel_dispatch_re()) {
        names.extend(resolve_php_queue_name(&raw, &uses, namespace.as_deref()));
    }
    names.sort();
    names.dedup();
    names
}

fn resolve_php_queue_name(name: &str, uses: &[String], namespace: Option<&str>) -> Vec<String> {
    let mut names = vec![name.to_string()];
    if let Some((_, short)) = name.rsplit_once('.') {
        names.push(short.to_string());
    }
    for import in uses {
        let target = import
            .split_once('=')
            .map_or(import.as_str(), |(_, target)| target);
        let alias = import.split_once('=').map(|(alias, _)| alias);
        if alias == Some(name)
            || target == name
            || target.ends_with(&format!(".{name}"))
            || target.rsplit('.').next() == Some(name)
        {
            names.push(target.to_string());
        }
    }
    if !name.contains('.') {
        if let Some(namespace) = namespace {
            names.push(format!("{namespace}.{name}"));
        }
    }
    names
}

pub(super) fn laravel_queue_identities(classes: &[String]) -> Vec<String> {
    let qualified: Vec<String> = classes
        .iter()
        .filter(|name| name.contains('.'))
        .cloned()
        .collect();
    if qualified.is_empty() {
        classes.to_vec()
    } else {
        qualified
    }
}

fn laravel_dispatch_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\b([A-Za-z_\\][A-Za-z0-9_\\]*)::dispatch\s*\(").expect("dispatch")
    })
}

pub(super) fn php_should_queue_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\bimplements\s+[^{;]*\bShouldQueue\b").expect("shouldqueue"))
}

pub(super) fn extract_messenger_dispatches(source: &str) -> Vec<String> {
    extract_named(source, messenger_dispatch_re())
}

pub(super) fn extract_messenger_workers(source: &str) -> Vec<String> {
    let mut names = extract_named(source, messenger_as_handler_re());
    names.extend(extract_named(source, messenger_interface_re()));
    names.extend(extract_named(source, messenger_invoke_re()));
    names.sort();
    names.dedup();
    names
}

fn messenger_dispatch_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\bdispatch\(\s*new\s+([A-Za-z_\\][A-Za-z0-9_\\]*)")
            .expect("messenger dispatch")
    })
}

fn messenger_as_handler_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"#\[AsMessageHandler[^\]]*\]\s*(?:final\s+)?class\s+([A-Za-z_][A-Za-z0-9_]*)")
            .expect("as handler")
    })
}

fn messenger_interface_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"class\s+([A-Za-z_][A-Za-z0-9_]*)\s+implements\s+[^{]*\bMessageHandlerInterface\b",
        )
        .expect("handler interface")
    })
}

fn messenger_invoke_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"function\s+__invoke\s*\(\s*([A-Za-z_\\][A-Za-z0-9_\\]*)").expect("invoke")
    })
}
