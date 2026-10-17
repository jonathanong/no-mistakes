// Exact bab collector recursion; only the actual container-entry probe is added.
use super::{Evaluator, FxHashSet, Path, PathBuf, Value};
pub(super) fn collect(
    evaluator: &Evaluator<'_, fn(&str, &Path) -> Option<PathBuf>>,
    value: &Value,
    ids: &mut FxHashSet<u64>,
    visits: &mut usize,
) {
    match value {
        Value::Prefix(_, _, Some(id)) => {
            ids.insert(*id);
        }
        Value::Promise(value) | Value::Evaluated(value, _) => {
            collect(evaluator, value, ids, visits)
        }
        Value::Aggregate(values)
        | Value::References(values)
        | Value::Object(values)
        | Value::Record(values, _)
        | Value::Joined(values)
        | Value::Possible(values) => {
            *visits += 1;
            for value in values {
                collect(evaluator, value, ids, visits);
            }
        }
        Value::Arguments(id) | Value::Array(id) if ids.insert(*id) => {
            // Container escape can replace live slots. Definite deletion
            // disconnects the former value; possible deletion does not.
            for (index, value) in evaluator.argument_slots(*id) {
                if !evaluator
                    .definite_deleted_argument_slots
                    .contains(&(*id, index))
                {
                    collect(evaluator, value, ids, visits);
                }
            }
        }
        _ => {}
    }
}
