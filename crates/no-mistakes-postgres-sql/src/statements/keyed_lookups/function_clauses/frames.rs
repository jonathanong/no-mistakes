use super::super::conditions::{decimal_integer, literal};
use super::{is_known_aggregate, visit_eager_functions};
use sqlparser::ast::{Expr, Function, Value, WindowFrameBound, WindowFrameUnits, WindowType};

pub(in crate::statements::keyed_lookups) fn empty_aggregate_frame(function: &Function) -> bool {
    let Some(WindowType::WindowSpec(spec)) = &function.over else {
        return false;
    };
    spec.window_frame.as_ref().is_some_and(|frame| {
        frame.units == WindowFrameUnits::Rows
            && is_known_aggregate(function)
            && relative_row(&frame.start_bound)
                .zip(relative_row(
                    frame
                        .end_bound
                        .as_ref()
                        .unwrap_or(&WindowFrameBound::CurrentRow),
                ))
                .is_some_and(|(start, end)| start > end)
    })
}

pub(in crate::statements::keyed_lookups) fn visit_function_filter(
    function: &Function,
    visit: &mut impl FnMut(&Expr),
) {
    if let Some(filter) = &function.filter {
        if empty_aggregate_frame(function) {
            visit_eager_functions(filter, visit);
        } else {
            visit(filter);
        }
    }
}

fn relative_row(bound: &WindowFrameBound) -> Option<i64> {
    let (expression, direction) = match bound {
        WindowFrameBound::CurrentRow => return Some(0),
        WindowFrameBound::Preceding(expression) => (expression, -1),
        WindowFrameBound::Following(expression) => (expression, 1),
    };
    let Value::Number(number, _) = literal(expression.as_deref()?)? else {
        return None;
    };
    let offset = decimal_integer(number)?;
    (offset >= 0).then(|| direction * offset)
}
