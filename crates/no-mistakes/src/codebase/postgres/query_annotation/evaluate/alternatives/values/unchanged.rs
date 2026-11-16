use super::Value;

pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn unchanged(
    before: &Value,
    after: &Value,
) -> bool {
    match (before, after) {
        (Value::Prefix(text, complete, id), Value::Prefix(next, done, next_id)) => {
            text == next && complete == done && id == next_id
        }
        (Value::Promise(before), Value::Promise(after))
        | (Value::Evaluated(before, _), Value::Evaluated(after, _)) => unchanged(before, after),
        (Value::Aggregate(before), Value::Aggregate(after))
        | (Value::References(before), Value::References(after))
        | (Value::Object(before), Value::Object(after))
        | (Value::Record(before, _), Value::Record(after, _))
        | (Value::Joined(before), Value::Joined(after))
        | (Value::Possible(before), Value::Possible(after)) => {
            before.identity() == after.identity() && !before.contains_arguments()
        }
        _ => !before.contains_reference(),
    }
}
