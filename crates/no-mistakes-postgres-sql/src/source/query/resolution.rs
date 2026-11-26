use super::*;
use sqlparser::ast::{Expr, ObjectName};
impl Collector<'_, '_> {
    pub(super) fn column(
        &self,
        expr: &Expr,
        scope: usize,
        clause: PostgresSqlQueryClause,
    ) -> Option<PostgresSqlQueryColumn> {
        let sql_name = match expr {
            Expr::Identifier(id) => ObjectName::from(vec![id.clone()]),
            Expr::CompoundIdentifier(ids) => ObjectName::from(ids.clone()),
            Expr::Nested(inner) => return self.column(inner, scope, clause),
            _ => return None,
        };
        let name = name(&sql_name);
        let keys: Vec<_> = name.parts.iter().map(|p| p.identity.clone()).collect();
        let mut result = PostgresSqlQueryColumn {
            scope_id: scope,
            clause,
            name,
            relation_id: None,
            relation_scope_id: None,
            resolution: PostgresSqlQueryColumnResolution::Unqualified,
            span: self.locations.span(expr.span()),
        };
        if keys.len() < 2 {
            return Some(result);
        }
        result.resolution = PostgresSqlQueryColumnResolution::Unknown;
        let mut current = Some(scope);
        while let Some(id) = current {
            if let Some(relations) = self.states[id].names.get(&keys[..keys.len() - 1]) {
                if relations.len() == 1 {
                    result.relation_id = Some(relations[0]);
                    result.relation_scope_id = Some(id);
                    result.resolution = PostgresSqlQueryColumnResolution::Resolved;
                } else {
                    result.resolution = PostgresSqlQueryColumnResolution::Ambiguous;
                }
                break;
            }
            current = self.states[id].visible_parent;
        }
        Some(result)
    }
    pub(super) fn register(&mut self, relation: PostgresSqlQueryRelation) -> usize {
        let id = relation.id;
        let mut keys = Vec::new();
        if let Some(alias) = &relation.alias {
            keys.push(vec![alias.identity.clone()]);
        } else if let Some(name) = &relation.name {
            let full: Vec<_> = name.parts.iter().map(|p| p.identity.clone()).collect();
            if let Some(last) = full.last() {
                keys.push(vec![last.clone()]);
            }
            if full.len() > 1 {
                keys.push(full);
            }
        }
        for key in keys {
            self.states[relation.scope_id]
                .names
                .entry(key)
                .or_default()
                .push(id);
        }
        self.facts.relations.push(relation);
        id
    }
}
