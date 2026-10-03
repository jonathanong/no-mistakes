//! Temporary relation identity follows SQL source order, never crossing source boundaries.
use super::items::sql_name;
use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::statements::{
    SqlBoundFact, SqlBoundItemKind, SqlBoundQuery, SqlPinSource,
};
use sqlparser::ast::{Expr, ObjectName, ObjectNamePart, ObjectType, SetExpr, Statement};
use std::collections::BTreeSet;

#[derive(Default)]
pub(in super::super) struct TemporaryRelations(BTreeSet<String>);

impl TemporaryRelations {
    pub(in super::super) fn apply(&mut self, statement: &Statement, facts: &mut [SqlBoundFact]) {
        // SELECT INTO's source is resolved before its destination is created.
        for fact in facts {
            self.query(&mut fact.query);
        }
        match statement {
            Statement::CreateTable(table) if table.temporary => {
                self.insert(sql_name(&table.name));
            }
            Statement::CreateView(view) if view.temporary => {
                self.insert(sql_name(&view.name));
            }
            Statement::Query(query) => {
                if let SetExpr::Select(select) = &*query.body {
                    if let Some(into) = &select.into {
                        if into.temporary {
                            for target in &into.targets {
                                let parts = match unwrap_expr(target) {
                                    Expr::Identifier(ident) => {
                                        vec![ObjectNamePart::Identifier(ident.clone())]
                                    }
                                    Expr::CompoundIdentifier(idents) => idents
                                        .iter()
                                        .cloned()
                                        .map(ObjectNamePart::Identifier)
                                        .collect(),
                                    _ => continue,
                                };
                                self.insert(sql_name(&ObjectName(parts)));
                            }
                        }
                    }
                }
            }
            Statement::Drop {
                object_type: ObjectType::Table | ObjectType::View,
                names,
                ..
            } => {
                for name in names {
                    let name = sql_name(name);
                    self.0
                        .remove(name.strip_prefix("pg_temp.").unwrap_or(&name));
                }
            }
            _ => {}
        }
    }

    fn insert(&mut self, name: String) {
        self.0
            .insert(name.strip_prefix("pg_temp.").unwrap_or(&name).to_string());
    }

    fn query(&self, query: &mut SqlBoundQuery) {
        for item in &mut query.items {
            match &mut item.kind {
                SqlBoundItemKind::Table(name)
                    if self
                        .0
                        .contains(name.strip_prefix("pg_temp.").unwrap_or(name)) =>
                {
                    item.kind = SqlBoundItemKind::Opaque;
                    item.pins.clear();
                }
                SqlBoundItemKind::Query(query) => self.query(query),
                _ => {}
            }
            for pin in &mut item.pins {
                if let SqlPinSource::Query(query) = &mut pin.source {
                    self.query(query);
                }
            }
        }
    }
}
