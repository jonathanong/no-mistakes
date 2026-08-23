use super::references::reference_exists;
use super::{Options, RuleFinding, RULE_ID};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};

mod extract;
mod oxlint;
mod pnpm;
#[cfg(test)]
mod tests;
mod types;

pub(crate) use extract::{extract, matches_preset};
use types::Extracted;

struct ScanContext<'a> {
    root: &'a Path,
    config: &'a NoMistakesConfig,
    rel_files: &'a [String],
    sources: &'a crate::codebase::ts_source::SourceStore,
    findings: &'a mut Vec<RuleFinding>,
}

pub(super) fn scan(
    root: &Path,
    config: &NoMistakesConfig,
    opts: &Options,
    config_candidates: &[PathBuf],
    rel_files: &[String],
    sources: &crate::codebase::ts_source::SourceStore,
    findings: &mut Vec<RuleFinding>,
) -> Result<()> {
    if opts.presets.is_empty() {
        return Ok(());
    }
    let mut context = ScanContext {
        root,
        config,
        rel_files,
        sources,
        findings,
    };
    for path in config_candidates {
        let filename = config_filename(path);
        let rel = crate::codebase::ts_source::relative_slash_path(root, path);
        let matched: Vec<&str> = opts
            .presets
            .iter()
            .map(String::as_str)
            .filter(|preset| matches_preset(preset, &filename, &rel))
            .collect();
        if matched.is_empty() {
            continue;
        }
        scan_file(&mut context, path, &rel, &matched)?;
    }
    Ok(())
}

fn scan_file(
    context: &mut ScanContext<'_>,
    path: &Path,
    rel: &str,
    matched: &[&str],
) -> Result<()> {
    let Some(source) = super::super::read_source(context.sources, path) else {
        return Ok(());
    };
    for extracted in matched
        .iter()
        .filter(|preset| *preset == &"no-mistakes")
        .flat_map(|_| extract::no_mistakes(context.config))
    {
        push_missing(
            context.root,
            path,
            rel,
            context.rel_files,
            context.findings,
            extracted,
        )?;
    }
    for extracted in matched
        .iter()
        .filter(|preset| *preset == &"pnpm-workspace-filters")
        .flat_map(|_| pnpm::workspace_filters(&source))
    {
        push_missing(
            context.root,
            path,
            rel,
            context.rel_files,
            context.findings,
            extracted,
        )?;
    }
    let structured: Vec<&str> = matched
        .iter()
        .copied()
        .filter(|preset| *preset != "pnpm-workspace-filters" && *preset != "no-mistakes")
        .collect();
    if structured.is_empty() {
        return Ok(());
    }
    let value = match crate::codebase::structured_value::parse_structured_value(path, &source) {
        Ok(value) => value,
        Err(error) => {
            context.findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: rel.to_string(),
                line: 1,
                message: format!("{rel}: {error}"),
                import: None,
                target: None,
            });
            return Ok(());
        }
    };
    for preset in structured {
        for extracted in extract(preset, &value) {
            push_missing(
                context.root,
                path,
                rel,
                context.rel_files,
                context.findings,
                extracted,
            )?;
        }
    }
    Ok(())
}

fn push_missing(
    root: &Path,
    path: &Path,
    rel: &str,
    rel_files: &[String],
    findings: &mut Vec<RuleFinding>,
    extracted: Extracted,
) -> Result<()> {
    let opts = Options {
        allow_globs: extracted.allow_globs,
        base_dir: extracted.base_dir,
        ..Default::default()
    };
    if !reference_exists(root, path, &opts, &extracted.value, rel_files)? {
        findings.push(RuleFinding {
            rule: RULE_ID.to_string(),
            file: rel.to_string(),
            line: 1,
            message: format!(
                "{rel}: config path `{}` from `{}` does not exist",
                extracted.value, extracted.field
            ),
            import: None,
            target: Some(extracted.value),
        });
    }
    Ok(())
}

fn config_filename(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_string()
}
