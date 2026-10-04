use super::Collector;
use sqlparser::ast::{Query, SetExpr};

impl Collector<'_, '_, '_> {
    pub(super) fn collect_query(&mut self, query: &Query, outer: &[String]) {
        let mut ctes = outer.to_vec();
        if let Some(with) = &query.with {
            for cte in &with.cte_tables {
                let name = crate::codebase::postgres::idents::ident_key(&cte.alias.name);
                if with.recursive {
                    ctes.push(name.clone());
                }
                self.collect_query(&cte.query, &ctes);
                if !with.recursive {
                    ctes.push(name);
                }
            }
        }
        self.collect_set(&query.body, &ctes);
    }

    fn collect_set(&mut self, set: &SetExpr, ctes: &[String]) {
        match set {
            SetExpr::Update(statement)
            | SetExpr::Delete(statement)
            | SetExpr::Insert(statement)
            | SetExpr::Merge(statement) => self.collect_in_scope(statement, ctes),
            SetExpr::Query(query) => self.collect_query(query, ctes),
            SetExpr::SetOperation { left, right, .. } => {
                self.collect_set(left, ctes);
                self.collect_set(right, ctes);
            }
            _ => {}
        }
    }
}
