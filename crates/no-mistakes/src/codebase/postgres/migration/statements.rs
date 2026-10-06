use super::line_containing;
use crate::codebase::postgres::types::{SqlSchemaFileFacts, SqlStatementKind};
use sqlparser::ast::{ObjectType, Statement};

pub(super) fn record(sql: &str, statement: &Statement, facts: &mut SqlSchemaFileFacts) {
    let Some((kind, parts)) = kind_and_parts(statement) else {
        return;
    };
    facts.statement_kinds.push(SqlStatementKind {
        kind: kind.to_string(),
        line: line_containing(sql, parts),
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
pub(crate) fn policy_facts(sql: &str, statements: &[Statement]) -> SqlSchemaFileFacts {
    let mut facts = SqlSchemaFileFacts::default();
    for statement in statements {
        record(sql, statement, &mut facts);
    }
    record_new(sql, &mut facts);
    for dynamic in super::dynamic::schema_bodies(sql)
        .into_iter()
        .chain(super::dynamic::extract(sql))
    {
        let parsed = crate::codebase::postgres::parse::parse_postgres_sql_lenient(&dynamic.sql);
        let mut nested = policy_facts(&dynamic.sql, &parsed);
        super::dynamic::remap_fact_lines(&mut nested, &dynamic);
        facts.statement_kinds.extend(nested.statement_kinds);
        facts.setting_uses.extend(nested.setting_uses);
    }
    facts
}
