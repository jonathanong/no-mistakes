use super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::{OffsetUse, SqlOffsetFact};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    facts: &CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for path in files {
        let rel = relative_slash_path(root, path);
        let profile = (!rel.ends_with(".sql")).then_some(&opts.embedded);
        let statements = match facts.postgres_statements(path, profile) {
            Ok(statements) => statements,
            Err(_) if rel.ends_with(".sql") && facts.postgres.sql_source_not_found(path) => {
                continue
            }
            Err(error) => return Err(error),
        };
        let mut ordinal = 0;
        for statement in statements {
            for offset in &statement.offset_uses {
                ordinal += 1;
                findings.push(finding(&rel, offset, ordinal));
            }
        }
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}

fn finding(file: &str, offset: &SqlOffsetFact, ordinal: usize) -> RuleFinding {
    let line = offset.line.max(1);
    let text = match offset.kind {
        OffsetUse::Zero => "OFFSET 0 used as an optimizer fence; use a MATERIALIZED CTE (WITH x AS MATERIALIZED (...))",
        OffsetUse::Other => "do not use SQL OFFSET; use cursor pagination, LIMIT + 1, COUNT, EXISTS, or ROW_NUMBER() instead",
    };
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!("{file}:{line}: {text}"),
        import: None,
        target: Some(if ordinal == 1 {
            "offset".to_string()
        } else {
            format!("offset#{ordinal}")
        }),
    }
}
