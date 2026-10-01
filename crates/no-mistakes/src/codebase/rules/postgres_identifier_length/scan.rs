use super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::collect_postgres_facts;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    let facts = collect_postgres_facts(
        root,
        sources,
        files,
        &CheckFactPlan {
            postgres_schema: true,
            ..CheckFactPlan::default()
        },
        &opts.schema,
        &Default::default(),
    )
    .map_err(|error| anyhow::anyhow!("{RULE_ID} option sqlInclude: {error}"))?;
    let mut findings = Vec::new();
    for file in &facts.schema {
        let rel = relative_slash_path(root, &file.path);
        for identifier in &file.declared_identifiers {
            if identifier.name.len() <= opts.max_bytes {
                continue;
            }
            findings.push(finding(
                &rel,
                identifier.line,
                &identifier.name,
                opts.max_bytes,
            ));
        }
    }
    Ok(findings)
}

fn finding(file: &str, line: usize, name: &str, max_bytes: usize) -> RuleFinding {
    let line = line.max(1);
    let truncated = truncate_identifier(name, max_bytes);
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!(
            "{file}:{line}: identifier \"{name}\" is {} bytes; PostgreSQL truncates identifiers longer than {max_bytes} bytes to \"{truncated}\"",
            name.len()
        ),
        import: None,
        target: Some(name.to_string()),
    }
}

fn truncate_identifier(name: &str, max_bytes: usize) -> String {
    if name.len() <= max_bytes {
        return name.to_string();
    }
    let mut end = max_bytes.min(name.len());
    while end > 0 && !name.is_char_boundary(end) {
        end -= 1;
    }
    name[..end].to_string()
}
