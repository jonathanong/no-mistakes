use super::inputs::input_depends_on_data_at;
use super::PlaceholderPositions;
use crate::idents::visit_function_args;
use crate::statements::SqlBoundItemKind;
use sqlparser::ast::{Expr, FunctionArg};

/// Named-argument labels are syntax. `name := value` is rewritten to a named
/// argument before this walk, so only the value is a data input.
pub(super) fn caller_supplied_argument(
    arg: &FunctionArg,
    positions: PlaceholderPositions<'_>,
) -> bool {
    let mut supplied = false;
    visit_function_args(std::slice::from_ref(arg), &mut |expr| {
        supplied = !input_depends_on_data_at(expr, positions);
    });
    supplied
}

/// `unnest(…)` as a table factor is sized by its arrays, unless one is taken from a query or
/// from another FROM item (the table factor is then evaluated per row of that item).
pub(in crate::statements::bounds) fn unnest_kind_at(
    arrays: &[Expr],
    positions: PlaceholderPositions<'_>,
) -> SqlBoundItemKind {
    if arrays
        .iter()
        .any(|expr| input_depends_on_data_at(expr, positions))
    {
        SqlBoundItemKind::Opaque
    } else {
        SqlBoundItemKind::Other
    }
}
