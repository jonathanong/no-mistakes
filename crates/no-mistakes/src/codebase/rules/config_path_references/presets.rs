use super::references::reference_exists;
use super::{Options, RuleFinding, RULE_ID};
use anyhow::Result;
use std::path::{Path, PathBuf};

mod extract;
mod oxlint;
mod types;

#[cfg(test)]
pub(crate) use extract::pnpm_workspace_filters;
pub(crate) use extract::{extract, matches_preset};
use types::Extracted;

pub(super) fn scan(
    root: &Path,
    opts: &Options,
    config_candidates: &[PathBuf],
    rel_files: &[String],
    sources: &crate::codebase::ts_source::SourceStore,
    findings: &mut Vec<RuleFinding>,
) -> Result<()> {
    if opts.presets.is_empty() {
        return Ok(());
    }
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
        scan_file(root, path, &rel, &matched, rel_files, sources, findings)?;
    }
    Ok(())
}

fn scan_file(
    root: &Path,
    path: &Path,
    rel: &str,
    matched: &[&str],
    rel_files: &[String],
    sources: &crate::codebase::ts_source::SourceStore,
    findings: &mut Vec<RuleFinding>,
) -> Result<()> {
    let Some(source) = super::super::read_source(sources, path) else {
        return Ok(());
    };
    for extracted in matched
        .iter()
        .filter(|preset| *preset == &"pnpm-workspace-filters")
        .flat_map(|_| extract::pnpm_workspace_filters(&source))
    {
        push_missing(root, path, rel, rel_files, findings, extracted)?;
    }
    let structured: Vec<&str> = matched
        .iter()
        .copied()
        .filter(|preset| *preset != "pnpm-workspace-filters")
        .collect();
    if structured.is_empty() {
        return Ok(());
    }
    let value = match crate::codebase::structured_value::parse_structured_value(path, &source) {
        Ok(value) => value,
        Err(error) => {
            findings.push(RuleFinding {
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
            push_missing(root, path, rel, rel_files, findings, extracted)?;
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
