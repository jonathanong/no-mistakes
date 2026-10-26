use crate::codebase::postgres::types::{SqlSchemaFileFacts, SqlStatementKind};
use sqlparser::ast::{ObjectType, Statement};

pub(super) fn record(line: usize, statement: &Statement, facts: &mut SqlSchemaFileFacts) {
    super::super::function_calls::collect(statement, &mut facts.function_calls);
    let Some((kind, _parts)) = kind_and_parts(statement) else {
        return;
    };
    facts.statement_kinds.push(SqlStatementKind {
        kind: kind.to_string(),
        line,
    });
}

fn kind_and_parts(statement: &Statement) -> Option<(&'static str, &'static [&'static str])> {
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
        _ => None,
    }
}

mod policy;
pub(super) use policy::record as record_new;
pub(crate) use policy::SUPPORTED_KINDS;

/// Reuse the prepared AST and the existing recursive routine projection.
pub(crate) fn policy_facts(
    sql: &str,
    statements: &[Statement],
    tokens: &[&[sqlparser::tokenizer::TokenWithSpan]],
) -> SqlSchemaFileFacts {
    let mut facts = SqlSchemaFileFacts::default();
    for (index, statement) in statements.iter().enumerate() {
        let locations =
            super::locating::Locations::new(tokens.get(index).copied().unwrap_or_default());
        record(locations.statement_line(statement), statement, &mut facts);
    }
    record_new(sql, &mut facts);
    for dynamic in super::dynamic::schema_bodies(sql)
        .into_iter()
        .chain(super::dynamic::extract(sql))
    {
        let (located, functions) = crate::codebase::postgres::parse::partition_function_sources(
            crate::codebase::postgres::parse::parse_postgres_sql_with_function_sources(
                &dynamic.sql,
            ),
        );
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
        let mut nested = policy_facts(&dynamic.sql, &parsed, &tokens);
        nested.function_calls.extend(functions);
        super::dynamic::remap_fact_lines(&mut nested, &dynamic);
        facts.statement_kinds.extend(nested.statement_kinds);
        facts.setting_uses.extend(nested.setting_uses);
        facts.function_calls.extend(nested.function_calls);
    }
    facts
}
