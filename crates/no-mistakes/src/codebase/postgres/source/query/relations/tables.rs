use super::*;
use sqlparser::ast::TableFactor;
impl Collector<'_, '_> {
    pub(super) fn relation(
        &mut self,
        table: &TableFactor,
        scope: usize,
        env: &CteEnvironment,
    ) -> Vec<usize> {
        use PostgresSqlQueryRelationKind as K;
        let mut relation = PostgresSqlQueryRelation {
            id: self.facts.relations.len(),
            scope_id: scope,
            kind: K::Unsupported,
            name: None,
            alias: None,
            column_aliases: Vec::new(),
            cte_id: None,
            subquery_scope_id: None,
            members: Vec::new(),
            lateral: false,
            span: self.locations.span(table.span()),
        };
        let alias = match table {
            TableFactor::Table {
                name: table_name,
                alias,
                args,
                sample,
                ..
            } => {
                let n = name(table_name);
                if args.is_none() {
                    relation.cte_id = if n.parts.len() == 1 {
                        env.get(&n.parts[0].identity).copied()
                    } else {
                        None
                    };
                    relation.kind = if relation.cte_id.is_some() {
                        K::Cte
                    } else {
                        K::Table
                    };
                } else {
                    self.unsupported(
                        scope,
                        PostgresSqlQueryClause::From,
                        "table function",
                        table.span(),
                    );
                }
                if sample.is_some() {
                    self.unsupported(
                        scope,
                        PostgresSqlQueryClause::From,
                        "table sample",
                        table.span(),
                    );
                }
                relation.name = Some(n);
                alias.as_ref()
            }
            TableFactor::Derived {
                lateral,
                subquery,
                alias,
                sample,
            } => {
                relation.kind = K::Derived;
                relation.lateral = *lateral;
                let visible = if *lateral {
                    Some(scope)
                } else {
                    self.states[scope].visible_parent
                };
                relation.subquery_scope_id = Some(self.query(
                    subquery,
                    Some(scope),
                    visible,
                    PostgresSqlQueryClause::From,
                    env,
                    self.facts.scopes[scope].cte_definition_id,
                ));
                // Child relations have consumed IDs before this containing relation.
                relation.id = self.facts.relations.len();
                if sample.is_some() {
                    self.unsupported(
                        scope,
                        PostgresSqlQueryClause::From,
                        "derived table sample",
                        table.span(),
                    );
                }
                alias.as_ref()
            }
            TableFactor::NestedJoin {
                table_with_joins,
                alias,
            } => {
                let before = self.states[scope].names.clone();
                let members = self.from(table_with_joins, scope, env);
                if alias.is_none() {
                    return members;
                }
                self.states[scope].names = before;
                relation.kind = K::Joined;
                relation.members = members;
                relation.id = self.facts.relations.len();
                alias.as_ref()
            }
            _ => {
                self.unsupported(
                    scope,
                    PostgresSqlQueryClause::From,
                    "relation form",
                    table.span(),
                );
                None
            }
        };
        if let Some(alias) = alias {
            relation.alias = Some(identifier(&alias.name));
            relation.column_aliases = alias.columns.iter().map(|c| identifier(&c.name)).collect();
        }
        vec![self.register(relation)]
    }
}
