use super::super::{expressions::identifier, types::*};
use sqlparser::ast::{Expr, Value};

pub(super) fn provenance(
    expr: &Expr,
    table: Option<&PostgresSqlName>,
    alias: Option<&PostgresSqlIdentifier>,
) -> PostgresSqlInsertProvenance {
    use PostgresSqlInsertProvenance as P;
    match expr {
        Expr::Nested(inner) | Expr::Cast { expr: inner, .. } => provenance(inner, table, alias),
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
        _ => P::Unresolved,
    }
}
