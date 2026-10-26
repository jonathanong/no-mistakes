//! Array cardinality follows input rows, never the aggregate's single output row.
use super::super::Scope;
use super::{array::scalar_subquery, subquery_source, Resolver, Sourced};
use crate::codebase::postgres::idents::{object_name_ident, unwrap_expr};
use crate::codebase::postgres::statements::SqlPinSource;
use sqlparser::ast::{
    DataType, Expr, FunctionArg, FunctionArgExpr, FunctionArguments, SelectItem, SetExpr,
};

pub(super) fn source(
    expr: &Expr,
    resolver: &Resolver,
    scope: &Scope,
    positions: super::super::super::value::PlaceholderPositions<'_>,
) -> Option<Sourced> {
    let query = match unwrap_expr(expr) {
        Expr::Function(function) if builtin(&function.name, "array") => {
            let FunctionArguments::Subquery(query) = &function.args else {
                return None;
            };
            query.as_ref()
        }
        Expr::Cast {
            expr,
            data_type: DataType::Array(_),
            ..
        } => {
            return aggregate(scalar_subquery(expr)?, resolver, scope, positions);
        }
        _ => return None,
    };
    let mut source = subquery_source(query, resolver, scope, positions);
    if let SqlPinSource::Query(query) = source.source {
        source.source = SqlPinSource::ArrayQuery(query);
    }
    Some(source)
}

fn aggregate(
    query: &sqlparser::ast::Query,
    resolver: &Resolver,
    scope: &Scope,
    positions: super::super::super::value::PlaceholderPositions<'_>,
) -> Option<Sourced> {
    let SetExpr::Select(select) = &*query.body else {
        return None;
    };
    if select.having.is_some()
        || !matches!(&select.group_by, sqlparser::ast::GroupByExpr::Expressions(expressions, _) if expressions.is_empty())
    {
        return None;
    }
    let [SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. }] =
        select.projection.as_slice()
    else {
        return None;
    };
    let Expr::Function(function) = unwrap_expr(expr) else {
        return None;
    };
    if !builtin(&function.name, "array_agg")
        || function.over.is_some()
        || function.filter.is_some()
        || !function.within_group.is_empty()
    {
        return None;
    }
    let FunctionArguments::List(arguments) = &function.args else {
        return None;
    };
    if !arguments.clauses.is_empty() {
        return None;
    }
    let [FunctionArg::Unnamed(FunctionArgExpr::Expr(element))] = arguments.args.as_slice() else {
        return None;
    };
    // The SELECT was established above; retain that invariant without rematching a clone.
    let mut input_select = select.clone();
    input_select.projection = vec![SelectItem::UnnamedExpr(element.clone())];
    let mut input = query.clone();
    *input.body = SetExpr::Select(input_select);
    // LIMIT on an aggregate sizes output groups, not the rows accumulated into each array.
    input.limit_clause = None;
    input.fetch = None;
    input.order_by = None;
    let reads = resolver.reads(query);
    let bound = super::super::query::bound_query(&input, scope, positions);
    Some(Sourced {
        source: if reads.certain {
            SqlPinSource::ReadQuery(bound)
        } else {
            SqlPinSource::ArrayQuery(bound)
        },
        reads: reads.bare,
        qualified_reads: reads.qualified,
    })
}

fn builtin(name: &sqlparser::ast::ObjectName, expected: &str) -> bool {
    let qualified = match name.0.as_slice() {
        [_] => true,
        [schema, _] => schema.as_ident().is_some_and(|schema| {
            schema.quote_style.is_none() && schema.value.eq_ignore_ascii_case("pg_catalog")
        }),
        _ => false,
    };
    qualified
        && object_name_ident(name).is_some_and(|ident| {
            ident.value.eq_ignore_ascii_case(expected) && ident.quote_style.is_none()
        })
}
