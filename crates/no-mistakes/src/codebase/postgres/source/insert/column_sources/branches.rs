use super::super::super::types::*;
use sqlparser::ast::{Expr, SelectItem, SetExpr, SetQuantifier};

pub(super) enum Branch<'a> {
    Values {
        path: Vec<usize>,
        rows: &'a [sqlparser::ast::Parens<Vec<Expr>>],
    },
    Select {
        path: Vec<usize>,
        expressions: Vec<&'a Expr>,
    },
}

pub(super) fn collect<'a>(
    body: &'a SetExpr,
    path: Vec<usize>,
    depth: usize,
    branches: &mut Vec<Branch<'a>>,
) -> Result<(), PostgresSqlInsertColumnSources> {
    if depth >= 64 {
        return Err(unsupported_details(
            PostgresSqlInsertColumnSourcesReason::NestingLimit,
            Some(path),
            None,
            None,
            None,
        ));
    }
    match body {
        SetExpr::Select(select) => {
            let mut expressions = Vec::with_capacity(select.projection.len());
            for item in &select.projection {
                match item {
                    SelectItem::UnnamedExpr(expr) | SelectItem::ExprWithAlias { expr, .. } => {
                        expressions.push(expr)
                    }
                    SelectItem::ExprWithAliases { .. } => {
                        return Err(unsupported_details(
                            PostgresSqlInsertColumnSourcesReason::AliasExpansion,
                            Some(path.clone()),
                            None,
                            None,
                            None,
                        ));
                    }
                    SelectItem::Wildcard(_) | SelectItem::QualifiedWildcard(_, _) => {
                        return Err(unsupported_details(
                            PostgresSqlInsertColumnSourcesReason::WildcardProjection,
                            Some(path.clone()),
                            None,
                            None,
                            None,
                        ));
                    }
                }
            }
            if expressions.is_empty() {
                return Err(unsupported(
                    PostgresSqlInsertColumnSourcesReason::EmptySource,
                ));
            }
            branches.push(Branch::Select { path, expressions });
        }
        SetExpr::Query(query) => collect(&query.body, path, depth + 1, branches)?,
        SetExpr::SetOperation {
            left,
            right,
            set_quantifier,
            ..
        } => {
            if matches!(
                set_quantifier,
                SetQuantifier::ByName | SetQuantifier::AllByName | SetQuantifier::DistinctByName
            ) {
                return Err(unsupported_details(
                    PostgresSqlInsertColumnSourcesReason::SetOperationByName,
                    Some(path),
                    None,
                    None,
                    None,
                ));
            }
            collect(left, branch_path(&path, 0), depth + 1, branches)?;
            collect(right, branch_path(&path, 1), depth + 1, branches)?;
        }
        SetExpr::Values(values) => branches.push(Branch::Values {
            path,
            rows: &values.rows,
        }),
        _ => {
            return Err(unsupported(
                PostgresSqlInsertColumnSourcesReason::UnsupportedSource,
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_arity(
    columns: usize,
    branches: &[Branch<'_>],
) -> Option<PostgresSqlInsertColumnSources> {
    for branch in branches {
        let (path, rows) = match branch {
            Branch::Values { path, rows } => {
                if rows.is_empty() {
                    return Some(unsupported_details(
                        PostgresSqlInsertColumnSourcesReason::EmptySource,
                        Some(path.clone()),
                        None,
                        Some(columns),
                        Some(0),
                    ));
                }
                for (row_index, row) in rows.iter().enumerate() {
                    if row.len() != columns {
                        return Some(unsupported_details(
                            PostgresSqlInsertColumnSourcesReason::SourceArityMismatch,
                            Some(path.clone()),
                            Some(row_index),
                            Some(columns),
                            Some(row.len()),
                        ));
                    }
                }
                (path, None)
            }
            Branch::Select { path, expressions } => (path, Some(expressions.len())),
        };
        if rows.is_some_and(|actual| actual != columns) {
            return Some(unsupported_details(
                PostgresSqlInsertColumnSourcesReason::SourceArityMismatch,
                Some(path.clone()),
                None,
                Some(columns),
                rows,
            ));
        }
    }
    None
}

fn branch_path(path: &[usize], branch: usize) -> Vec<usize> {
    let mut child = path.to_vec();
    child.push(branch);
    child
}

pub(super) fn unsupported(
    reason: PostgresSqlInsertColumnSourcesReason,
) -> PostgresSqlInsertColumnSources {
    unsupported_details(reason, None, None, None, None)
}

fn unsupported_details(
    reason: PostgresSqlInsertColumnSourcesReason,
    branch_path: Option<Vec<usize>>,
    row_index: Option<usize>,
    expected_columns: Option<usize>,
    source_columns: Option<usize>,
) -> PostgresSqlInsertColumnSources {
    PostgresSqlInsertColumnSources::Unsupported {
        reason,
        branch_path,
        row_index,
        expected_columns,
        source_columns,
    }
}
