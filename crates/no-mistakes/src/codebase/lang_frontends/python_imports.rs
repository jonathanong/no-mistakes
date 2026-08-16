use super::super::facts::module_from_path;
use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

pub(super) fn python_module(
    package: Option<&str>,
    package_root: Option<&Path>,
    path: &Path,
) -> Option<String> {
    let package = package_prefix(package?)?;
    match package_root.and_then(|root| module_from_path(root, path)) {
        Some(rel) => Some(match package {
            Some(package) => format!("{package}.{rel}"),
            None => rel,
        }),
        None => package.map(str::to_string),
    }
}

fn package_prefix(package: &str) -> Option<Option<&str>> {
    let trimmed = package.trim();
    if trimmed.is_empty() || trimmed == "." {
        return Some(None);
    }
    Some(Some(trimmed))
}

pub(super) fn prefix_package(package: Option<&str>, module: String) -> String {
    match package.and_then(|name| package_prefix(name).flatten()) {
        Some(package) => format!("{package}.{module}"),
        None => module,
    }
}

pub(super) fn extract_python_imports(
    source: &str,
    path: &Path,
    package: Option<&str>,
    package_root: Option<&Path>,
) -> Vec<String> {
    let source = super::super::strip::mask_strings(source);
    let mut imports = Vec::new();
    for raw in extract_named(&source, python_import_re()) {
        for part in raw.split(',') {
            let mut tokens = part.split_whitespace();
            if let Some(name) = tokens.next() {
                imports.push(name.to_string());
                if tokens
                    .next()
                    .is_some_and(|token| token.eq_ignore_ascii_case("as"))
                {
                    if let Some(alias) = tokens.next() {
                        imports.push(format!("{alias}={name}"));
                    }
                }
            }
        }
    }
    for cap in python_from_re().captures_iter(&source) {
        let Some(module) = cap.get(1).map(|m| m.as_str()) else {
            continue;
        };
        let names = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        if let Some(resolved) = resolve_relative(module, path, package_root) {
            let resolved = prefix_package(package, resolved);
            if module.chars().all(|ch| ch == '.') {
                push_imported_members(&mut imports, &resolved, names);
            } else {
                imports.push(resolved.clone());
                push_imported_members(&mut imports, &resolved, names);
            }
        } else if !module.starts_with('.') {
            imports.push(module.to_string());
            push_imported_members(&mut imports, module, names);
        }
    }
    imports.sort();
    imports.dedup();
    imports
}

fn push_imported_members(imports: &mut Vec<String>, module: &str, names: &str) {
    for (name, alias) in imported_bindings(names) {
        let qualified = format!("{module}.{name}");
        imports.push(qualified.clone());
        if let Some(alias) = alias {
            imports.push(format!("{alias}={qualified}"));
        }
    }
}

fn imported_bindings(names: &str) -> Vec<(String, Option<String>)> {
    names
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .split(',')
        .filter_map(|part| {
            let mut tokens = part.split_whitespace();
            let ident = tokens.next()?;
            if ident.is_empty() || ident.starts_with('(') || ident == "*" {
                return None;
            }
            let alias = tokens
                .next()
                .filter(|token| token.eq_ignore_ascii_case("as"))
                .and_then(|_| tokens.next())
                .map(str::to_string);
            Some((ident.to_string(), alias))
        })
        .collect()
}

fn resolve_relative(module: &str, path: &Path, package_root: Option<&Path>) -> Option<String> {
    let dots = module.chars().take_while(|ch| *ch == '.').count();
    if dots == 0 {
        return None;
    }
    let rest = module[dots..].trim_matches('.');
    let mut dir = path.parent()?.to_path_buf();
    for _ in 1..dots {
        dir = dir.parent()?.to_path_buf();
    }
    let package_root = package_root?;
    let target = if rest.is_empty() {
        dir
    } else {
        dir.join(rest.replace('.', std::path::MAIN_SEPARATOR_STR))
    };
    module_from_path(package_root, &target.with_extension("py"))
        .or_else(|| module_from_path(package_root, &target.join("__init__.py")))
}

fn extract_named(source: &str, re: &Regex) -> Vec<String> {
    let mut values: Vec<String> = re
        .captures_iter(source)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect();
    values.sort();
    values.dedup();
    values
}

fn python_import_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?m)^\s*import\s+([^\n]+)").expect("python import"))
}

fn python_from_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?ms)^\s*from\s+(\.+(?:[A-Za-z_][\w.]*)?|[A-Za-z_][\w.]*)\s+import\s+(\([^)]+\)|[^\n]+)",
        )
        .expect("from")
    })
}
