use super::super::SqlSelectFact;
use super::{from, nested, shapes, stars, uses};
use sqlparser::ast::{OrderByExpr, Query, Select, SetExpr, Statement};

type PlaceholderPositions<'a> = super::super::value::PlaceholderPositions<'a>;

struct Collector<'sql, 'positions, 'out> {
    sql: &'sql str,
    positions: PlaceholderPositions<'positions>,
    out: &'out mut Vec<SqlSelectFact>,
}

pub(in crate::codebase::postgres::statements) fn collect_query_at(
    sql: &str,
    query: &Query,
    outer_ctes: &[String],
    in_insert_select: bool,
    in_exists: bool,
    positions: super::super::value::PlaceholderPositions<'_>,
    out: &mut Vec<SqlSelectFact>,
) {
    let mut ctes = outer_ctes.to_vec();
    if let Some(with) = &query.with {
        for cte in &with.cte_tables {
            let name = crate::codebase::postgres::idents::ident_key(&cte.alias.name);
            if with.recursive {
                ctes.push(name.clone());
            }
            collect_query_at(
                sql,
                &cte.query,
                &ctes,
                in_insert_select,
                false,
                positions,
                out,
            );
            if !with.recursive {
                ctes.push(name);
            }
        }
    }
    nested::collect_query_expressions_at(sql, query, &ctes, in_insert_select, positions, out);
    let order = query_order(query);
    Collector {
        sql,
        positions,
        out,
    }
    .collect_set(&query.body, &ctes, in_insert_select, in_exists, order);
}

fn query_order(query: &Query) -> &[sqlparser::ast::OrderByExpr] {
    match query.order_by.as_ref().map(|order| &order.kind) {
        Some(sqlparser::ast::OrderByKind::Expressions(exprs)) => exprs,
        _ => &[],
    }
}

impl Collector<'_, '_, '_> {
    fn collect_set(
        &mut self,
        expr: &SetExpr,
        ctes: &[String],
        in_insert_select: bool,
        in_exists: bool,
        order: &[OrderByExpr],
    ) {
        match expr {
            SetExpr::Select(select) => {
                self.push_select(select, ctes, in_insert_select, in_exists, order)
            }
            SetExpr::Query(query) => collect_query_at(
                self.sql,
                query,
                ctes,
                in_insert_select,
                in_exists,
                self.positions,
                self.out,
            ),
            SetExpr::SetOperation { left, right, .. } => {
                self.collect_set(left, ctes, in_insert_select, in_exists, &[]);
                self.collect_set(right, ctes, in_insert_select, in_exists, &[]);
            }
            SetExpr::Values(values) => {
                for expr in values.rows.iter().flat_map(|row| row.iter()) {
                    nested::walk_expr_at(
                        self.sql,
                        expr,
                        ctes,
                        in_insert_select,
                        self.positions,
                        self.out,
                    );
                }
            }
            SetExpr::Insert(Statement::Insert(insert)) => {
                if let Some(source) = insert.source.as_deref() {
                    collect_query_at(
                        self.sql,
                        source,
                        ctes,
                        true,
                        false,
                        self.positions,
                        self.out,
                    );
                }
            }
            _ => {}
        }
    }

    fn push_select(
        &mut self,
        select: &Select,
        ctes: &[String],
        in_insert_select: bool,
        in_exists: bool,
        order: &[OrderByExpr],
    ) {
        let sql = self.sql;
        let positions = self.positions;
        let out = &mut self.out;
        let tables = from::table_names(&select.from, ctes);
        let mut exists_set_operations = Vec::new();
        super::super::exists::collect_from_select_at(
            sql,
            select,
            positions,
            &mut exists_set_operations,
        );
        from::collect_derived_queries(sql, &select.from, ctes, in_insert_select, positions, out);
        nested::collect_at(sql, select, ctes, in_insert_select, positions, out);
        let relations = super::super::predicates::select_relations(sql, select, ctes, positions);
        let shapes = shapes::collect(select);
        let line = super::line::select_line(sql, select, &tables);
        let star_projections = if in_exists {
            Vec::new()
        } else {
            stars::collect(select, ctes, line)
        };
        let column_uses = uses::collect(select, order, ctes, line);
        if tables.is_empty()
            && exists_set_operations.is_empty()
            && relations.is_empty()
            && shapes.not_in_subqueries.is_empty()
            && shapes.count_existence_checks.is_empty()
            && star_projections.is_empty()
            && column_uses.is_empty()
        {
            return;
        }
        out.push(SqlSelectFact {
            line,
            tables,
            predicate_sql: from::predicate_text(select),
            exists_set_operations,
            relations,
            in_insert_select,
            not_in_subqueries: shapes.not_in_subqueries,
            not_in_columns: shapes.not_in_columns,
            count_existence_checks: shapes.count_existence_checks,
            star_projections,
            column_uses,
        });
    }
}
