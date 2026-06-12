use super::Options;
use crate::codebase::ts_source::TS_JS_EXTENSIONS;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug)]
pub(super) struct SourceInfo {
    pub(super) rel: String,
    pub(super) dir: String,
    pub(super) stem: String,
    pub(super) base: String,
    pub(super) import_path: String,
}

pub(super) fn source_extensions(opts: &Options) -> HashSet<String> {
    if opts.source_extensions.is_empty() {
        TS_JS_EXTENSIONS
            .iter()
            .map(|ext| format!(".{ext}"))
            .collect()
    } else {
        opts.source_extensions
            .iter()
            .map(|ext| {
                if ext.starts_with('.') {
                    ext.clone()
                } else {
                    format!(".{ext}")
                }
            })
            .collect()
    }
}

pub(super) fn source_info(
    rel: &str,
    opts: &Options,
    source_globs: Option<&GlobSet>,
    extensions: &HashSet<String>,
    exclude_basenames: &HashSet<&str>,
) -> Option<SourceInfo> {
    let extension = extensions
        .iter()
        .find(|extension| rel.ends_with(extension.as_str()))?;
    if is_declaration_file(rel) {
        return None;
    }
    if source_globs.is_some_and(|globs| !globs.is_match(rel)) {
        return None;
    }
    let (dir, base) = split_dir_base(rel);
    if !opts.source_dirs.is_empty()
        && !opts
            .source_dirs
            .iter()
            .any(|source_dir| source_dir_matches(&dir, source_dir, opts.direct_child_only))
    {
        return None;
    }
    if exclude_basenames.contains(base.as_str())
        || opts
            .exclude_prefixes
            .iter()
            .any(|prefix| base.starts_with(prefix))
    {
        return None;
    }
    let stem = base.strip_suffix(extension.as_str())?.to_string();
    let source_path = rel.strip_suffix(extension.as_str())?.to_string();
    let import_path = if opts.strip_source_prefix.is_empty() {
        source_path.clone()
    } else {
        source_path
            .strip_prefix(opts.strip_source_prefix.trim_start_matches('/'))
            .unwrap_or(source_path.as_str())
            .trim_start_matches('/')
            .to_string()
    };
    Some(SourceInfo {
        rel: rel.to_string(),
        dir,
        stem,
        base,
        import_path,
    })
}

pub(super) fn source_dir_matches(dir: &str, source_dir: &str, direct_child_only: bool) -> bool {
    let source_dir = source_dir.trim_matches('/');
    if source_dir.is_empty() {
        return false;
    }
    if direct_child_only {
        dir == source_dir
    } else {
        dir == source_dir || dir.starts_with(&format!("{source_dir}/"))
    }
}

pub(super) fn split_dir_base(rel: &str) -> (String, String) {
    match rel.rfind('/') {
        Some(index) => (rel[..index].to_string(), rel[index + 1..].to_string()),
        None => (String::new(), rel.to_string()),
    }
}

pub(super) fn build_globset(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(Some(builder.build()?))
}

pub(super) fn build_companion_globset(opts: &Options, source: &SourceInfo) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in &opts.companion_globs {
        builder.add(Glob::new(&render_glob_template(pattern, source))?);
    }
    Ok(builder.build()?)
}

fn render_glob_template(template: &str, source: &SourceInfo) -> String {
    template
        .replace("{sourcePath}", &glob_escape_literal(&source.import_path))
        .replace("{sourceRel}", &glob_escape_literal(&source.rel))
        .replace("{sourceDir}", &glob_escape_literal(&source.dir))
        .replace("{sourceStem}", &glob_escape_literal(&source.stem))
        .replace("{sourceBase}", &glob_escape_literal(&source.base))
}

pub(super) fn render_template(template: &str, source: &SourceInfo) -> String {
    template
        .replace("{sourcePath}", &source.import_path)
        .replace("{sourceRel}", &source.rel)
        .replace("{sourceDir}", &source.dir)
        .replace("{sourceStem}", &source.stem)
        .replace("{sourceBase}", &source.base)
}

pub(super) fn file_imports(root: &Path, rel: &str, expected_specifier: &str) -> bool {
    let Ok(source) = std::fs::read_to_string(root.join(rel)) else {
        return false;
    };
    source.contains(&format!("from \"{expected_specifier}\""))
        || source.contains(&format!("from '{expected_specifier}'"))
        || source.contains(&format!("import \"{expected_specifier}\""))
        || source.contains(&format!("import '{expected_specifier}'"))
}

fn glob_escape_literal(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| {
            if matches!(ch, '*' | '?' | '[' | ']' | '{' | '}' | '\\') {
                vec!['\\', ch]
            } else {
                vec![ch]
            }
        })
        .collect()
}

fn is_declaration_file(rel: &str) -> bool {
    rel.ends_with(".d.ts") || rel.ends_with(".d.mts") || rel.ends_with(".d.cts")
}
