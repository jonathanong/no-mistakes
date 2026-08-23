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
