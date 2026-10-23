use super::CompiledOptions;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::{parse_postgres_sql, EmbeddedSqlCall, EmbeddedSqlKind};
use crate::codebase::ts_source::relative_slash_path;
use sqlparser::ast::{ObjectName, ObjectNamePart, Visit, Visitor};
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};

pub(crate) struct SqlCall<'a> {
    pub file: PathBuf,
    pub span: (u32, u32),
    pub call: &'a EmbeddedSqlCall,
}

pub(crate) fn sql_calls<'a>(
    root: &Path,
    opts: &CompiledOptions,
    facts: &'a CheckFactMap,
) -> Vec<SqlCall<'a>> {
    let mut out = Vec::new();
    for path in facts.graph_file_universe() {
        let rel = relative_slash_path(root, path);
        if !opts.includes(&rel) {
            continue;
        }
        let Ok(sql) = facts.embedded_sql(path, &opts.embedded) else {
            continue;
        };
        for (call, span) in sql.calls.iter().zip(sql.call_spans.iter()) {
            if sql
                .unproven_member_query_spans
                .iter()
                .any(|other| overlapping(*span, *other))
            {
                continue;
            }
            out.push(SqlCall {
                file: path.clone(),
                span: *span,
                call,
            });
        }
    }
    out
}

pub(crate) fn overlapping(left: (u32, u32), right: (u32, u32)) -> bool {
    left.0 < right.1 && right.0 < left.1
}

/// `Some(true)` when a recovered relation interpolates `sql_placeholder_N`.
/// Parse failure is unknown provenance, not a violation.
pub(crate) fn identifier_placeholder(call: &EmbeddedSqlCall) -> Option<bool> {
    if call.kind == EmbeddedSqlKind::Dynamic && call.sql_text.is_none() && call.variants.is_empty()
    {
        return None;
    }
    let mut texts: Vec<&str> = call
        .variants
        .iter()
        .map(|variant| variant.sql_text.as_str())
        .collect();
    if texts.is_empty() {
        texts.extend(call.sql_text.as_deref());
    }
    if texts.is_empty() {
        return None;
    }
    let mut unknown = call.variants_truncated();
    let mut found = false;
    for sql in texts {
        match relation_placeholder(sql) {
            None => unknown = true,
            Some(true) => found = true,
            Some(false) => {}
        }
    }
    if found {
        Some(true)
    } else if unknown {
        None
    } else {
        Some(false)
    }
}

fn relation_placeholder(sql: &str) -> Option<bool> {
    let statements = parse_postgres_sql(sql).ok()?;
    let mut finder = RelationPlaceholder { found: false };
    for statement in &statements {
        let _ = statement.visit(&mut finder);
    }
    Some(finder.found)
}

struct RelationPlaceholder {
    found: bool,
}

impl Visitor for RelationPlaceholder {
    type Break = ();
    fn pre_visit_relation(&mut self, relation: &ObjectName) -> ControlFlow<()> {
        if object_has_placeholder(relation) {
            self.found = true;
        }
        ControlFlow::Continue(())
    }
}

fn object_has_placeholder(name: &ObjectName) -> bool {
    name.0.iter().any(|part| match part {
        ObjectNamePart::Identifier(ident) => ident.value.starts_with("sql_placeholder_"),
        _ => false,
    })
}

#[cfg(test)]
mod tests;
