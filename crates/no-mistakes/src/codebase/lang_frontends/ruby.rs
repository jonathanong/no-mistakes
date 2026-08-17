use super::facts::{configured_roots, files_under, owning_package, LangFactMap, LangFileFacts};
use super::strip::strip_comments_keep_strings;
use crate::codebase::ts_source::SourceStore;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub(crate) fn collect_ruby_facts(
    root: &Path,
    all_files: &[PathBuf],
    apps: &[String],
    sources: &SourceStore,
) -> LangFactMap {
    let roots = configured_roots(root, apps);
    let files = files_under(all_files, &roots, "rb");
    super::facts::collect_files_parallel(files, |path| parse_ruby_file(path, &roots, apps, sources))
}

fn parse_ruby_file(
    path: &Path,
    roots: &[PathBuf],
    apps: &[String],
    sources: &SourceStore,
) -> Option<LangFileFacts> {
    let source = sources.read_path(path).ok()?;
    let text = strip_comments_keep_strings(&source);
    Some(LangFileFacts {
        path: path.to_path_buf(),
        package: owning_package(path, roots, apps),
        module: ruby_module_key(path, roots).or_else(|| {
            path.file_stem()
                .map(|name| name.to_string_lossy().into_owned())
        }),
        imports: extract_requires(&text, path, roots),
        declarations: extract_ruby_declarations(&text),
        references: extract_named(&text, ruby_const_re()),
        route_handlers: extract_pairs(&text, rails_route_re()),
        queue_enqueues: extract_named(&text, active_job_re()),
        queue_workers: extract_named(&text, ruby_job_class_re()),
        mods: Vec::new(),
    })
}

fn ruby_module_key(path: &Path, roots: &[PathBuf]) -> Option<String> {
    let root = roots
        .iter()
        .filter(|candidate| path.starts_with(candidate))
        .max_by_key(|candidate| candidate.components().count())?;
    let rel = path.strip_prefix(root).ok()?;
    let key = rel.to_string_lossy().replace('\\', "/");
    Some(key.trim_end_matches(".rb").to_string())
}

fn extract_requires(source: &str, path: &Path, roots: &[PathBuf]) -> Vec<String> {
    let mut imports = extract_named(source, ruby_require_re());
    for rel in extract_named(source, ruby_require_relative_re()) {
        if let Some(parent) = path.parent() {
            let resolved = crate::codebase::ts_resolver::normalize_path(
                &parent.join(rel).with_extension("rb"),
            );
            if let Some(key) = ruby_module_key(&resolved, roots) {
                imports.push(key);
            }
            if let Some(stem) = resolved.file_stem() {
                imports.push(stem.to_string_lossy().into_owned());
            }
        }
    }
    imports.sort();
    imports.dedup();
    imports
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

fn extract_pairs(source: &str, re: &Regex) -> Vec<(String, String)> {
    re.captures_iter(source)
        .filter_map(|cap| {
            Some((
                cap.get(1)?.as_str().to_string(),
                cap.get(2)?.as_str().to_string(),
            ))
        })
        .collect()
}

fn ruby_require_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"\brequire\s+["']([^"']+)["']"#).expect("require"))
}

fn ruby_require_relative_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"\brequire_relative\s+["']([^"']+)["']"#).expect("rel"))
}

fn extract_ruby_declarations(source: &str) -> Vec<String> {
    let mut stack: Vec<(usize, String)> = Vec::new();
    let mut names = Vec::new();
    for line in source.lines() {
        let indent = line.chars().take_while(|ch| ch.is_whitespace()).count();
        while stack.last().is_some_and(|(depth, _)| *depth >= indent) {
            stack.pop();
        }
        if let Some(name) = ruby_decl_re()
            .captures(line)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
        {
            let qualified = if stack.is_empty() || name.contains("::") {
                name.clone()
            } else {
                format!(
                    "{}::{name}",
                    stack
                        .iter()
                        .map(|(_, part)| part.as_str())
                        .collect::<Vec<_>>()
                        .join("::")
                )
            };
            names.push(qualified);
            names.push(name.clone());
            stack.push((indent, name));
        }
    }
    names.sort();
    names.dedup();
    names
}

fn ruby_decl_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?m)^\s*(?:class|module)\s+([A-Z][\w:]*)").expect("decl"))
}

fn ruby_const_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\b([A-Z][A-Za-z0-9_]*(?:::[A-Z][A-Za-z0-9_]*)*)\b").expect("const")
    })
}

fn rails_route_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*(?:get|post|put|patch|delete)\s+["']([^"']+)["']\s*,\s*to:\s*["']([^"']+)["']"#)
            .expect("route")
    })
}

fn active_job_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b([A-Z][\w:]*)\.perform_later\b").expect("job"))
}

fn ruby_job_class_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?m)^\s*class\s+([A-Z][\w:]*)\s*<\s*ApplicationJob").expect("job class")
    })
}
