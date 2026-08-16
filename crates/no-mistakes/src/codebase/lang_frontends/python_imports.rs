use super::super::facts::module_from_path;
use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

pub(super) fn python_module(
    package: Option<&str>,
    package_root: Option<&Path>,
    path: &Path,
) -> Option<String> {
    let package = package?;
    match package_root.and_then(|root| module_from_path(root, path)) {
        Some(rel) => Some(format!("{package}.{rel}")),
        None => Some(package.to_string()),
    }
}

fn prefix_package(package: Option<&str>, module: String) -> String {
    match package {
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
    let mut imports = Vec::new();
    for raw in extract_named(source, python_import_re()) {
        for part in raw.split(',') {
            if let Some(name) = part.split_whitespace().next() {
                imports.push(name.to_string());
            }
        }
    }
    for cap in python_from_re().captures_iter(source) {
        let Some(module) = cap.get(1).map(|m| m.as_str()) else {
            continue;
        };
        let names = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        if let Some(resolved) = resolve_relative(module, path, package_root) {
            let resolved = prefix_package(package, resolved);
            if module.chars().all(|ch| ch == '.') {
                for name in imported_names(names) {
                    imports.push(format!("{resolved}.{name}"));
                }
            } else {
                imports.push(resolved);
            }
        } else if !module.starts_with('.') {
            imports.push(module.to_string());
            for name in imported_names(names) {
                imports.push(format!("{module}.{name}"));
            }
        }
    }
    imports.sort();
    imports.dedup();
    imports
}

fn imported_names(names: &str) -> Vec<String> {
    names
        .split(',')
        .filter_map(|part| {
            let ident = part.split_whitespace().next()?;
            if ident.is_empty() || ident.starts_with('(') || ident == "*" {
                return None;
            }
            Some(ident.to_string())
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
        Regex::new(r"(?m)^\s*from\s+(\.+(?:[A-Za-z_][\w.]*)?|[A-Za-z_][\w.]*)\s+import\s+([^\n]+)")
            .expect("from")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn extract_python_imports_covers_unprefixed_and_star_forms() {
        let path = Path::new("/repo/app/users/views.py");
        let imports = extract_python_imports(
            "import app.tasks, app.models\nfrom . import *\nfrom ...outside import nope\nfrom app.mod import helper",
            path,
            None,
            None,
        );
        assert!(imports.iter().any(|import| import == "app.tasks"));
        assert!(imports.iter().any(|import| import == "app.mod.helper"));
        assert_eq!(python_module(None, None, path), None);
        assert_eq!(
            prefix_package(None, "users.models".to_string()),
            "users.models"
        );
    }
}
