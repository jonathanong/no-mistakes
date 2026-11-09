use super::*;
pub(in super::super) fn cte_consumers(
    query: &Query,
    catalog: &super::super::scope::CatalogScope<'_>,
    names: &super::super::scope::definitions::Names,
    out: &mut Vec<Finding>,
) {
    let Some(with) = &query.with else { return };
    for cte in &with.cte_tables {
        let Some(inner) = super::super::expressions::select(&cte.query) else {
            continue;
        };
        let definition_scope = super::super::scope::CatalogScope {
            catalog: catalog.catalog,
            parameters: catalog.parameters,
            shadowed: names
                .get(&super::super::scope::definitions::key(&cte.query))
                .expect("CTE definition has a lexical scope"),
        };
        if !super::super::evidence::catalogued(cte.query.as_ref(), &definition_scope)
            || inner.distinct != Some(Distinct::Distinct)
            || !cte.alias.columns.is_empty()
            || cte.query.limit_clause.is_some()
            || cte.query.fetch.is_some()
        {
            continue;
        }
        let references = super::references::cte(query, cte);
        let total = references.len();
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
                let [from] = select.from.as_slice() else {
                    return;
                };
                if !from.joins.is_empty() {
                    return;
                }
                let Some(relation) = walk::relation(&from.relation) else {
                    return;
                };
                if !references.contains(&super::references::factor_key(&from.relation))
                    || !select.projection.iter().all(|item| match item {
                        SelectItem::Wildcard(options) => {
                            *options == WildcardAdditionalOptions::default()
                        }
                        SelectItem::QualifiedWildcard(
                            SelectItemQualifiedWildcardKind::ObjectName(owner),
                            options,
                        ) => {
                            *options == WildcardAdditionalOptions::default()
                                && crate::codebase::postgres::idents::object_name_key(owner)
                                    == relation.alias
                        }
                        SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
                            column(expr).is_some() || pinned_value(expr)
                        }
                        _ => false,
                    })
                {
                    return;
                }
                safe += 1;
            };
            match expr {
                Expr::InSubquery { subquery, .. } | Expr::Exists { subquery, .. } => {
                    inspect(subquery)
                }
                _ => {}
            }
            ControlFlow::Continue(())
        });
        if total > 0 && total == safe {
            out.push(finding(
                inner,
                "distinct-join-filter",
                "every CTE consumer tests membership or existence; remove the CTE DISTINCT",
            ));
        }
    }
}
