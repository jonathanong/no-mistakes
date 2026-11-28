use super::exprs::{arg_expr, walk_subqueries};
use super::items::collect_returning;
use super::{collect_query, collect_rels, push_factor, FragmentStar, Rel, Scope};
use no_mistakes_postgres_sql::idents::{ident_key, object_name_key};
use sqlparser::ast::{
    Delete, FromTable, FunctionArgExpr, Insert, JoinConstraint, JoinOperator, OnConflictAction,
    OnInsert, Table, TableFactor, TableFunctionArgs, TableObject, Update, UpdateTableFromKind,
};

pub(super) fn collect_insert(insert: &Insert, scope: Scope<'_>, out: &mut Vec<FragmentStar>) {
    if let TableObject::TableName(name) = &insert.table {
        let target = Rel {
            table: object_name_key(name),
            alias: insert.table_alias.as_ref().map(|a| ident_key(&a.alias)),
            join_aliases: Vec::new(),
            is_local_derived: false,
            depth: 0,
        };
        collect_returning(&target, &[], scope, insert.returning.as_deref(), out);
    }
    if let Some(source) = insert.source.as_deref() {
        collect_query(source, scope, false, out);
    }
    if let Some(OnInsert::OnConflict(conflict)) = &insert.on {
        if let OnConflictAction::DoUpdate(update) = &conflict.action {
            for assignment in &update.assignments {
                walk_subqueries(&assignment.value, scope, out);
            }
            if let Some(selection) = &update.selection {
                walk_subqueries(selection, scope, out);
            }
        }
    }
}

pub(super) fn collect_update(update: &Update, scope: Scope<'_>, out: &mut Vec<FragmentStar>) {
    let Some(target) = target_rel(&update.table.relation) else {
        return;
    };
    let from = match &update.from {
        Some(UpdateTableFromKind::BeforeSet(tables) | UpdateTableFromKind::AfterSet(tables)) => {
            tables.as_slice()
        }
        None => &[],
    };
    let mut peers = collect_rels(from, scope, out);
    for join in &update.table.joins {
        push_factor(&join.relation, scope, &mut peers, out);
        visit_join_constraint(&join.join_operator, scope, out);
    }
    for assignment in &update.assignments {
        walk_subqueries(&assignment.value, scope, out);
    }
    if let Some(selection) = &update.selection {
        walk_subqueries(selection, scope, out);
    }
    collect_returning(&target, &peers, scope, update.returning.as_deref(), out);
}

pub(super) fn collect_delete(delete: &Delete, scope: Scope<'_>, out: &mut Vec<FragmentStar>) {
    let tables = match &delete.from {
        FromTable::WithFromKeyword(t) | FromTable::WithoutKeyword(t) => t,
    };
    let Some(target) = tables.first().and_then(|table| target_rel(&table.relation)) else {
        return;
    };
    let peers = collect_rels(delete.using.as_deref().unwrap_or_default(), scope, out);
    delete
        .selection
        .iter()
        .for_each(|s| walk_subqueries(s, scope, out));
    collect_returning(&target, &peers, scope, delete.returning.as_deref(), out);
}

pub(super) fn collect_table_expr(table: &Table, ctes: &[String], out: &mut Vec<FragmentStar>) {
    let Some(name) = &table.table_name else {
        return;
    };
    let table_name = table.schema_name.as_ref().map_or_else(
        || ident_key(name),
        |schema| format!("{}.{}", ident_key(schema), ident_key(name)),
    );
    if !ctes.iter().any(|cte| cte == &table_name) {
        let line = (name.span.start.line as usize).max(1);
        out.push(FragmentStar {
            returning: false,
            qualifier: None,
            relation: Some(table_name),
            within_function: None,
            line,
        });
    }
}

pub(super) fn target_rel(factor: &TableFactor) -> Option<Rel> {
    let TableFactor::Table {
        name,
        alias,
        args: None,
        ..
    } = factor
    else {
        return None;
    };
    Some(Rel {
        table: object_name_key(name),
        alias: alias.as_ref().map(|a| ident_key(&a.name)),
        join_aliases: Vec::new(),
        is_local_derived: false,
        depth: 0,
    })
}

pub(super) fn visit_join_constraint(
    op: &JoinOperator,
    scope: Scope<'_>,
    out: &mut Vec<FragmentStar>,
) {
    let (JoinOperator::Join(JoinConstraint::On(expr))
    | JoinOperator::Inner(JoinConstraint::On(expr))
    | JoinOperator::Left(JoinConstraint::On(expr))
    | JoinOperator::LeftOuter(JoinConstraint::On(expr))
    | JoinOperator::Right(JoinConstraint::On(expr))
    | JoinOperator::RightOuter(JoinConstraint::On(expr))
    | JoinOperator::FullOuter(JoinConstraint::On(expr))) = op
    else {
        return;
    };
    walk_subqueries(expr, scope, out);
}

pub(super) fn visit_table_args(
    args: &TableFunctionArgs,
    scope: Scope<'_>,
    out: &mut Vec<FragmentStar>,
) {
    for arg in &args.args {
        if let FunctionArgExpr::Expr(e) = arg_expr(arg) {
            walk_subqueries(e, scope, out);
        }
    }
}
