use super::RuleFinding;
use crate::codebase::ts_source::{relative_slash_path, TS_JS_EXTENSIONS};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "required-companion-imports";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) source_dirs: Vec<String>,
    pub(crate) source_globs: Vec<String>,
    pub(crate) source_extensions: Vec<String>,
    pub(crate) direct_child_only: bool,
    pub(crate) exclude_basenames: Vec<String>,
    pub(crate) exclude_prefixes: Vec<String>,
    pub(crate) companion_globs: Vec<String>,
    pub(crate) specifier_template: String,
    pub(crate) strip_source_prefix: String,
}

#[derive(Debug)]
struct SourceInfo {
    rel: String,
    dir: String,
    stem: String,
    base: String,
    import_path: String,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.rule_options();
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &target_roots))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan(root, &opts, &files)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    if opts.companion_globs.is_empty() || opts.specifier_template.is_empty() {
        return Ok(Vec::new());
    }

    let source_globs = build_globset(&opts.source_globs)?;
    let extensions = source_extensions(opts);
    let exclude_basenames: HashSet<&str> =
        opts.exclude_basenames.iter().map(String::as_str).collect();
    let rel_files: Vec<String> = files
        .iter()
        .map(|path| relative_slash_path(root, path))
        .collect();

    let mut findings = Vec::new();
    for source in rel_files.iter().filter_map(|rel| {
        source_info(
            rel,
            opts,
            source_globs.as_ref(),
            &extensions,
            &exclude_basenames,
        )
    }) {
        let companion_globs = build_companion_globset(opts, &source)?;
        let companions = rel_files
            .iter()
            .filter(|rel| companion_globs.is_match(rel.as_str()))
            .collect::<Vec<_>>();
        let expected_specifier = render_template(&opts.specifier_template, &source);
        if companions.is_empty() {
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: source.rel.clone(),
                line: 1,
                message: format!(
                    "{}: no companion file found importing {}",
                    source.rel, expected_specifier
                ),
                import: None,
                target: Some(expected_specifier),
            });
            continue;
        }

        if !companions
            .iter()
            .any(|rel| file_imports(root, rel, &expected_specifier))
        {
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: source.rel.clone(),
                line: 1,
                message: format!(
                    "{}: companion files do not import {}",
                    source.rel, expected_specifier
                ),
                import: None,
                target: Some(expected_specifier),
            });
        }
    }
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}

fn source_extensions(opts: &Options) -> HashSet<String> {
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

fn source_info(
    rel: &str,
    opts: &Options,
    source_globs: Option<&GlobSet>,
    extensions: &HashSet<String>,
    exclude_basenames: &HashSet<&str>,
) -> Option<SourceInfo> {
    let extension = extensions
        .iter()
        .find(|extension| rel.ends_with(extension.as_str()))?;
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

fn source_dir_matches(dir: &str, source_dir: &str, direct_child_only: bool) -> bool {
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

fn split_dir_base(rel: &str) -> (String, String) {
    match rel.rfind('/') {
        Some(index) => (rel[..index].to_string(), rel[index + 1..].to_string()),
        None => (String::new(), rel.to_string()),
    }
}

fn build_globset(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(Some(builder.build()?))
}

fn build_companion_globset(opts: &Options, source: &SourceInfo) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in &opts.companion_globs {
        builder.add(Glob::new(&render_template(pattern, source))?);
    }
    Ok(builder.build()?)
}

fn render_template(template: &str, source: &SourceInfo) -> String {
    template
        .replace("{sourcePath}", &source.import_path)
        .replace("{sourceRel}", &source.rel)
        .replace("{sourceDir}", &source.dir)
        .replace("{sourceStem}", &source.stem)
        .replace("{sourceBase}", &source.base)
}

fn file_imports(root: &Path, rel: &str, expected_specifier: &str) -> bool {
    let Ok(source) = std::fs::read_to_string(root.join(rel)) else {
        return false;
    };
    source.contains(&format!("from \"{expected_specifier}\""))
        || source.contains(&format!("from '{expected_specifier}'"))
        || source.contains(&format!("import \"{expected_specifier}\""))
        || source.contains(&format!("import '{expected_specifier}'"))
}

#[cfg(test)]
mod tests;
