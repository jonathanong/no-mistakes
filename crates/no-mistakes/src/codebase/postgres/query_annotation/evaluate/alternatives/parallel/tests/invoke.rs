use super::*;
use std::path::{Path, PathBuf};
/// Invoke the saved returned closure as a known callee, before an opaque escape.
pub(super) fn callbacks<F: Fn(&str, &Path) -> Option<PathBuf>>(
    evaluator: &mut Evaluator<'_, F>,
    value: Value,
) {
    match value.exposed() {
        Value::Function(function, path, captured) => {
            let call = Expr::Call {
                callee: Box::new(Expr::Function(function)),
                args: Vec::new(),
                start: 0,
                end: 0,
            };
            evaluator.expr(&call, &path, &captured, 32, false);
        }
        Value::Joined(values) | Value::Aggregate(values) | Value::Possible(values) => {
            for value in values {
                callbacks(evaluator, value);
            }
        }
        _ => panic!("known saved callbacks must remain callable"),
    }
}
