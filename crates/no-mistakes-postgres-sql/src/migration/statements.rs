use crate::types::{SqlSchemaFileFacts, SqlStatementKind};
use sqlparser::ast::{ObjectType, Statement};
use sqlparser::tokenizer::Location;

mod clauses;

pub(super) fn record(
    line: usize,
    statement: &Statement,
    tokens: &[sqlparser::tokenizer::TokenWithSpan],
    procedural_body: bool,
    facts: &mut SqlSchemaFileFacts,
) {
    facts.clause_uses.extend(clauses::used(statement, tokens));
    super::super::function_calls::collect_executed(statement, &mut facts.function_calls);
    let Some((kind, _parts)) = kind_and_parts(statement, procedural_body) else {
        return;
    };
    facts.statement_kinds.push(SqlStatementKind {
        kind: kind.to_string(),
        line,
    });
}

fn kind_and_parts(
    statement: &Statement,
    procedural_body: bool,
) -> Option<(&'static str, &'static [&'static str])> {
    match statement {
        Statement::CreateTable(_) => Some(("CREATE TABLE", &["create", "table"])),
        Statement::AlterTable(_) => Some(("ALTER TABLE", &["alter", "table"])),
        Statement::CreateIndex(_) => Some(("CREATE INDEX", &["create", "index"])),
        Statement::CreateView(_) => Some(("CREATE VIEW", &["create", "view"])),
        Statement::Truncate(_) => Some(("TRUNCATE", &["truncate"])),
        Statement::Drop {
            object_type: ObjectType::Index,
            ..
        } => Some(("DROP INDEX", &["drop", "index"])),
        Statement::Drop {
            object_type: ObjectType::View | ObjectType::MaterializedView,
            ..
        } => Some(("DROP VIEW", &["drop", "view"])),
        Statement::StartTransaction {
            begin: true,
            statements,
            has_end_keyword,
            ..
        } if !procedural_body && statements.is_empty() && !has_end_keyword => {
            Some(("BEGIN", &["begin"]))
        }
        Statement::StartTransaction {
            begin: false,
            statements,
            has_end_keyword,
            ..
        } if statements.is_empty() && !has_end_keyword => {
            Some(("START TRANSACTION", &["start", "transaction"]))
        }
        Statement::Commit { end: false, .. } => Some(("COMMIT", &["commit"])),
        Statement::Commit { end: true, .. } if !procedural_body => Some(("END", &["end"])),
        Statement::Rollback {
            savepoint: None, ..
        } => Some(("ROLLBACK", &["rollback"])),
        Statement::Rollback {
            savepoint: Some(_), ..
        } => Some(("ROLLBACK TO SAVEPOINT", &["rollback", "to", "savepoint"])),
        Statement::Savepoint { .. } => Some(("SAVEPOINT", &["savepoint"])),
        Statement::ReleaseSavepoint { .. } => {
            Some(("RELEASE SAVEPOINT", &["release", "savepoint"]))
        }
        _ => None,
    }
}

mod policy;
pub(super) use policy::record as record_new;
pub use policy::{SUPPORTED_KINDS, TRANSACTION_CONTROL_KINDS};

/// Reuse the prepared AST and the existing recursive routine projection.
pub fn policy_facts(
    sql: &str,
    statements: &[Statement],
    tokens: &[&[sqlparser::tokenizer::TokenWithSpan]],
) -> SqlSchemaFileFacts {
    policy_facts_inner(sql, statements, tokens, None, false)
}

pub(crate) fn policy_facts_with_locations(
    sql: &str,
    statements: &[Statement],
    tokens: &[&[sqlparser::tokenizer::TokenWithSpan]],
    statement_locations: Option<&[Location]>,
) -> SqlSchemaFileFacts {
    policy_facts_inner(sql, statements, tokens, statement_locations, false)
}

fn policy_facts_inner(
    sql: &str,
    statements: &[Statement],
    tokens: &[&[sqlparser::tokenizer::TokenWithSpan]],
    statement_locations: Option<&[Location]>,
    procedural_body: bool,
) -> SqlSchemaFileFacts {
    let mut facts = SqlSchemaFileFacts::default();
    for (index, statement) in statements.iter().enumerate() {
        let tokens = tokens.get(index).copied().unwrap_or_default();
        let locations = super::locating::Locations::new(tokens);
        let fallback = statement_locations
            .and_then(|locs| locs.get(index))
            .copied();
        record(
            locations.statement_line(statement, fallback),
            statement,
            tokens,
            procedural_body,
            &mut facts,
        );
    }
    record_new(sql, &mut facts);
    let body_sources = super::dynamic::schema_bodies(sql)
        .into_iter()
        .map(|dynamic| (dynamic, true));
    let dynamic_sources = super::dynamic::extract(sql)
        .into_iter()
        .map(|dynamic| (dynamic, false));
    for (dynamic, is_body) in body_sources.chain(dynamic_sources) {
        let raw = if is_body {
            crate::parse::parse_postgres_body_with_function_sources(&dynamic.sql)
        } else {
            crate::parse::parse_postgres_sql_with_function_sources(&dynamic.sql)
        };
        let (located, functions) = crate::parse::partition_function_sources(raw);
        let sources: Vec<_> = located
            .iter()
            .map(|located| {
                located
                    .source
                    .clone()
                    .or_else(|| located.chunk_tokens.clone())
            })
            .collect();
        let tokens: Vec<_> = sources
            .iter()
            .map(|source| source.as_deref().unwrap_or_default())
            .collect();
        let parsed: Vec<_> = located
            .into_iter()
            .map(|located| located.statement)
            .collect();
        let mut nested = policy_facts_inner(&dynamic.sql, &parsed, &tokens, None, is_body);
        nested.function_calls.extend(functions);
        super::dynamic::remap_fact_lines(&mut nested, &dynamic);
        facts.statement_kinds.extend(nested.statement_kinds);
        facts.setting_uses.extend(nested.setting_uses);
        facts.clause_uses.extend(nested.clause_uses);
        facts.function_calls.extend(nested.function_calls);
    }
    facts
}
