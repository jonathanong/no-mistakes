use super::{SqlRelationPredicateFact, SqlSelectFact};

pub(super) fn collect_query(
    sql: &str,
    query: &sqlparser::ast::Query,
    outer: &[String],
    updates: &mut Vec<Vec<SqlRelationPredicateFact>>,
    deletes: &mut Vec<Vec<SqlRelationPredicateFact>>,
    selects: &mut Vec<SqlSelectFact>,
    column_uses: &mut Vec<super::super::SqlColumnUseFact>,
    positions: super::super::value::PlaceholderPositions<'_>,
) {
    let mut ctes = outer.to_vec();
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            let name = crate::codebase::postgres::idents::ident_key(&cte.alias.name);
            if with.recursive {
                ctes.push(name.clone());
            }
            collect_query(
                sql,
                &cte.query,
                &ctes,
                updates,
                deletes,
                selects,
                column_uses,
                positions,
            );
            if !with.recursive {
                ctes.push(name);
            }
        }
    }
    collect_set(
        sql,
        &query.body,
        &ctes,
        updates,
        deletes,
        selects,
        column_uses,
        positions,
    );
}

fn collect_set(
    sql: &str,
    set: &sqlparser::ast::SetExpr,
    ctes: &[String],
    updates: &mut Vec<Vec<SqlRelationPredicateFact>>,
    deletes: &mut Vec<Vec<SqlRelationPredicateFact>>,
    selects: &mut Vec<SqlSelectFact>,
    column_uses: &mut Vec<super::super::SqlColumnUseFact>,
    positions: super::super::value::PlaceholderPositions<'_>,
) {
    use sqlparser::ast::SetExpr;
    match set {
        SetExpr::Update(statement)
        | SetExpr::Delete(statement)
        | SetExpr::Insert(statement)
        | SetExpr::Merge(statement) => super::collect_in_scope(
            sql,
            statement,
            ctes,
            updates,
            deletes,
            selects,
            column_uses,
            positions,
        ),
        SetExpr::Query(query) => collect_query(
            sql,
            query,
            ctes,
            updates,
            deletes,
            selects,
            column_uses,
            positions,
        ),
        SetExpr::SetOperation { left, right, .. } => {
            collect_set(
                sql,
                left,
                ctes,
                updates,
                deletes,
                selects,
                column_uses,
                positions,
            );
            collect_set(
                sql,
                right,
                ctes,
                updates,
                deletes,
                selects,
                column_uses,
                positions,
            );
        }
        _ => {}
    }
}
