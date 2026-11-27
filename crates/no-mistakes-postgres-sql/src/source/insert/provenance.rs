use super::super::{expressions::identifier, types::*};
use sqlparser::ast::{Expr, Insert, UnaryOperator, Value};

pub(super) fn supported_modifiers(value: &Insert) -> bool {
    value.or.is_none()
        && !value.ignore
        && !value.overwrite
        && !value.has_table_keyword
        && value.assignments.is_empty()
        && value.partitioned.is_none()
        && value.after_columns.is_empty()
        && value.output.is_none()
        && !value.replace_into
        && value.priority.is_none()
        && value.insert_alias.is_none()
        && value.settings.is_none()
        && value.format_clause.is_none()
        && value.multi_table_insert_type.is_none()
        && value.multi_table_into_clauses.is_empty()
        && value.multi_table_when_clauses.is_empty()
        && value.multi_table_else_clause.is_none()
}

pub(super) fn provenance(
    expr: &Expr,
    table: Option<&PostgresSqlName>,
    alias: Option<&PostgresSqlIdentifier>,
) -> PostgresSqlInsertProvenance {
    use PostgresSqlInsertProvenance as P;
    match expr {
        Expr::Nested(inner) | Expr::Cast { expr: inner, .. } => provenance(inner, table, alias),
        Expr::UnaryOp {
            op: UnaryOperator::Plus | UnaryOperator::Minus,
            expr,
        } if matches!(expr.as_ref(), Expr::Value(value) if matches!(value.value, Value::Number(_, _))) => {
            P::Literal
        }
        Expr::Identifier(_) => P::TargetColumn,
        Expr::CompoundIdentifier(parts) if parts.len() >= 2 => {
            let qualifier = parts[..parts.len() - 1]
                .iter()
                .map(identifier)
                .collect::<Vec<_>>();
            if qualifier.len() == 1 && qualifier[0].identity == "excluded" {
                if alias.is_some_and(|alias| alias.identity == "excluded") {
                    P::Unresolved // The pseudo-relation and target alias collide.
                } else {
                    P::ExcludedColumn
                }
            } else if alias.is_some_and(|alias| qualifier == [alias.clone()])
                || alias.is_none()
                    && table.is_some_and(|table| {
                        qualifier == table.parts
                            || qualifier == table.parts[table.parts.len().saturating_sub(1)..]
                    })
            {
                P::TargetColumn
            } else {
                P::Unresolved
            }
        }
        Expr::Value(value) if matches!(value.value, Value::Placeholder(_)) => P::Placeholder,
        Expr::Value(_) | Expr::TypedString { .. } => P::Literal,
        Expr::Function(_) => P::Derived,
        _ => P::Unresolved,
    }
}

/// Completeness describes the exposed syntax, independently of lineage resolution.
pub(super) fn syntax_complete(root: &PostgresSqlExpressionRoot) -> bool {
    use PostgresSqlExpressionRoot as R;
    match root {
        R::ColumnReference { .. }
        | R::Literal { .. }
        | R::Parameter { .. }
        | R::TypedLiteral { .. } => true,
        R::Parenthesized { expression }
        | R::Cast { expression, .. }
        | R::Unary { expression, .. } => syntax_complete(expression),
        R::FunctionCall {
            arguments,
            arguments_complete,
            modifiers,
            ..
        } => {
            *arguments_complete
                && modifiers.is_empty()
                && arguments.iter().all(|arg| syntax_complete(&arg.root))
        }
        // These roots do not expose all operands as typed children.
        _ => false,
    }
}
