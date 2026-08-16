use super::facts::{configured_roots, files_under, owning_package, LangFactMap, LangFileFacts};
use super::strip::strip_comments_keep_strings;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub(crate) fn collect_go_facts(
    root: &Path,
    all_files: &[PathBuf],
    modules: &[String],
) -> LangFactMap {
    let roots = configured_roots(root, modules);
    let files = files_under(all_files, &roots, "go");
    let mut facts = LangFactMap::default();
    for path in files {
        if let Some(file) = parse_go_file(&path, &roots, modules) {
            facts.index_file(file);
        }
    }
    facts
}

fn parse_go_file(path: &Path, roots: &[PathBuf], modules: &[String]) -> Option<LangFileFacts> {
    let source = std::fs::read_to_string(path).ok()?;
    let text = strip_comments_keep_strings(&source);
    let package = owning_package(path, roots, modules);
    let module = go_import_path(path, roots);
    Some(LangFileFacts {
        path: path.to_path_buf(),
        package,
        module,
        imports: extract_go_imports(&text),
        declarations: extract_named(&text, go_decl_re()),
        references: extract_named(&text, go_ref_re()),
        route_handlers: Vec::new(),
        queue_enqueues: extract_named(&text, asynq_task_re()),
        queue_workers: extract_named(&text, asynq_handle_re()),
    })
}

fn go_import_path(path: &Path, roots: &[PathBuf]) -> Option<String> {
    let root = roots.iter().find(|candidate| path.starts_with(candidate))?;
    let module = std::fs::read_to_string(root.join("go.mod"))
        .ok()
        .and_then(|source| {
            source.lines().find_map(|line| {
                line.strip_prefix("module ")
                    .map(|value| value.trim().to_string())
            })
        });
    let rel = path.parent()?.strip_prefix(root).ok()?;
    let suffix = rel.to_string_lossy().replace('\\', "/");
    match (module, suffix.as_str()) {
        (Some(module), "") => Some(module),
        (Some(module), suffix) => Some(format!("{module}/{suffix}")),
        (None, "") => root
            .file_name()
            .map(|name| name.to_string_lossy().into_owned()),
        (None, suffix) => Some(suffix.to_string()),
    }
}

fn extract_go_imports(source: &str) -> Vec<String> {
    let mut imports = extract_named(source, go_single_import_re());
    if let Some(block) = go_import_block_re().captures(source) {
        imports.extend(extract_named(
            block.get(1).map(|m| m.as_str()).unwrap_or(""),
            go_quoted_re(),
        ));
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

fn go_single_import_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?m)^\s*import\s+"([^"]+)""#).expect("import"))
}

fn go_import_block_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?s)import\s*\((.*?)\)").expect("block"))
}

fn go_quoted_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#""([^"]+)""#).expect("quoted"))
}

fn go_decl_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?m)^\s*func\s+(?:\([^)]+\)\s+)?([A-Z][A-Za-z0-9_]*)").expect("func")
    })
}

fn go_ref_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b([A-Z][A-Za-z0-9_]*)\b").expect("ref"))
}

fn asynq_task_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"asynq\.NewTask\(\s*"([^"]+)""#).expect("task"))
}

fn asynq_handle_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"HandleFunc\(\s*"([^"]+)""#).expect("handle"))
}
