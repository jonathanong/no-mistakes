use super::{Scope, SqlBoundKind};
use sqlparser::ast::Statement;
use std::borrow::Cow;

impl Scope {
    pub(super) fn with_body_owner(&self, statement: Option<&Statement>) -> Cow<'_, Self> {
        if self.bodies.is_none() {
            return Cow::Borrowed(self);
        }
        let mut scope = self.clone();
        scope.body_owner = Some(statement.map_or(SqlBoundKind::Select, kind));
        Cow::Owned(scope)
    }
}

fn kind(statement: &Statement) -> SqlBoundKind {
    match statement {
        Statement::Update(_) => SqlBoundKind::Update,
        Statement::Delete(_) => SqlBoundKind::Delete,
        Statement::Query(query) => {
            super::query::modifying_statement(query).map_or(SqlBoundKind::Select, kind)
        }
        _ => SqlBoundKind::Select,
    }
}
