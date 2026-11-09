use super::*;
pub(in super::super) fn cte_consumers(
    query: &Query,
    catalog: &super::super::scope::CatalogScope<'_>,
    out: &mut Vec<Finding>,
) {
    let Some(with) = &query.with else { return };
    for cte in &with.cte_tables {
        let Some(inner) = super::super::expressions::select(&cte.query) else {
            continue;
        };
        if !super::super::evidence::catalogued(inner, catalog)
            || inner.distinct != Some(Distinct::Distinct)
            || !cte.alias.columns.is_empty()
            || cte.query.limit_clause.is_some()
            || cte.query.fetch.is_some()
        {
            continue;
        }
        let name = ident(&cte.alias.name);
        let total = references(query, &name);
        let mut safe = 0;
        let _: ControlFlow<()> = visit_expressions(query, |expr| {
            let mut inspect = |subquery: &Query| {
                let Some(select) = super::super::expressions::select(subquery) else {
                    return;
                };
                if subquery.limit_clause.is_some()
                    || subquery.fetch.is_some()
                    || !groups(select).is_empty()
                    || select.having.is_some()
                    || select.distinct.is_some()
                {
                    return;
                }
                let Some(exprs) = projection(select) else {
                    return;
                };
                if exprs
                    .iter()
                    .any(|expr| column(expr).is_none() && !pinned_value(expr))
                {
                    return;
                }
                let [from] = select.from.as_slice() else {
                    return;
                };
                if !from.joins.is_empty() {
                    return;
                }
                if let Some(relation) = walk::relation(&from.relation) {
                    if relation.cte.as_deref() == Some(&name) {
                        safe += 1;
                    }
                }
            };
            match expr {
                Expr::InSubquery { subquery, .. } | Expr::Exists { subquery, .. } => {
                    inspect(subquery)
                }
                _ => {}
            }
            ControlFlow::Continue(())
        });
        let other_consumers = with
            .cte_tables
            .iter()
            .filter(|other| other.alias.name != cte.alias.name)
            .any(|other| references(&other.query, &name) != 0);
        if total > 0 && total == safe && !other_consumers {
            out.push(finding(
                inner,
                "distinct-join-filter",
                "every CTE consumer tests membership or existence; remove the CTE DISTINCT",
            ));
        }
    }
}
fn references(node: &impl Visit, name: &str) -> usize {
    struct Count<'a> {
        name: &'a str,
        count: usize,
    }
    impl Visitor for Count<'_> {
        type Break = ();
        fn pre_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<()> {
            if walk::relation(factor)
                .is_some_and(|relation| relation.cte.as_deref() == Some(self.name))
            {
                self.count += 1;
            }
            ControlFlow::Continue(())
        }
    }
    let mut count = Count { name, count: 0 };
    let _ = node.visit(&mut count);
    count.count
}
