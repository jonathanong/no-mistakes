use super::{CompiledOptions, RULE_ID};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::{
    recovered_sql_may_select, EmbeddedSqlCall, OffsetUse, SqlOffsetFact,
};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::fx::FxHashMap;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    facts: &CheckFactMap,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    let mut dedup = crate::codebase::rules::VariantFindingDedup::default();
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
        let embedded = profile
            .map(|profile| facts.embedded_sql(path, profile))
            .transpose()?;
        let all_calls = embedded
            .into_iter()
            .flat_map(|file| file.calls.iter().enumerate())
            .flat_map(|(index, call)| {
                call.statement_calls()
                    .map(move |version| (index, !call.variants.is_empty(), version))
            });
        if opts.fail_unanalyzable {
            // Calls without recovered text have no statement facts to pair below.
            findings.extend(
                all_calls
                    .clone()
                    .filter(|(_, _, call)| call.is_unanalyzable() && call.sql_text.is_none())
                    .map(|(_, _, call)| unanalyzable(&rel, &call)),
            );
        }
        let mut calls = all_calls.filter(|(_, _, call)| call.sql_text.is_some());
        let source = profile.map(|_| sources.read_path(path)).transpose()?;
        let mut ordinal = 0;
        let mut variant_ordinals = FxHashMap::default();
        for statement in statements {
            let projected_call = calls.next();
            let call = projected_call.as_ref().map(|(_, _, call)| call.as_ref());
            // Only a verified prefix was recovered; an opaque tail could add OFFSET.
            let opaque_tail = opts.fail_unanalyzable && statement.offset_uses.is_empty();
            if let Some(call) = call.filter(|call| opaque_tail && dynamic_select_prefix(call)) {
                findings.push(unanalyzable(&rel, call));
            }
            let disabled_call = call.filter(|call| {
                // Calls exist only for embedded profiles, whose source was read above.
                let source = source.as_deref().unwrap();
                crate::codebase::ts_source::matching_disable_directive(
                    source,
                    Some(call.line),
                    RULE_ID,
                )
                .is_some()
            });
            for (offset_index, offset) in statement.offset_uses.iter().enumerate() {
                let site = crate::codebase::postgres::statements::SqlFactSite::Offset(offset_index);
                let position = statement
                    .variant_locations
                    .as_ref()
                    .and_then(|locations| locations.position(site.clone()));
                let occurrence = match position.and_then(|position| position.source_offset) {
                    Some(source_offset) => {
                        *variant_ordinals.entry(source_offset).or_insert_with(|| {
                            ordinal += 1;
                            ordinal
                        })
                    }
                    None => {
                        ordinal += 1;
                        ordinal
                    }
                };
                let mut offset = *offset;
                if let Some(call) = disabled_call {
                    // Preserve existing executor directives through the common suppression pass.
                    offset.line = call.line as usize;
                }
                dedup.push(
                    &mut findings,
                    statement,
                    site,
                    source.as_deref(),
                    finding(&rel, &offset, occurrence),
                );
            }
        }
    }
    crate::codebase::rules::sort_postgres_findings(&mut findings);
    Ok(findings)
}

fn dynamic_select_prefix(call: &EmbeddedSqlCall) -> bool {
    call.is_unanalyzable() && recovered_sql_may_select(call.sql_text.as_deref())
}

fn unanalyzable(file: &str, call: &EmbeddedSqlCall) -> RuleFinding {
    let line = call.line.max(1) as usize;
    RuleFinding {
        source_offset: None,
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!(
            "{file}:{line}: executed SQL is not statically recoverable, so an OFFSET in a dynamic SELECT cannot be ruled out; pass a SQL literal or trusted tagged template, or set unanalyzableSql: ignore"
        ),
        import: None,
        target: Some("unanalyzable".to_string()),
    }
}

fn finding(file: &str, offset: &SqlOffsetFact, ordinal: usize) -> RuleFinding {
    let line = offset.line.max(1);
    let text = match offset.kind {
        OffsetUse::Zero => "OFFSET 0 used as an optimizer fence; use a MATERIALIZED CTE (WITH x AS MATERIALIZED (...))",
        OffsetUse::Other => "do not use SQL OFFSET; use cursor pagination, LIMIT + 1, COUNT, EXISTS, or ROW_NUMBER() instead",
    };
    RuleFinding {
        source_offset: None,
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
