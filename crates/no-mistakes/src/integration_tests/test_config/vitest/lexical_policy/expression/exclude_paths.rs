use super::{Kind, Value};

// A path that omits exclude keeps the earlier literal, or stays omitted.
pub(super) fn partial_exclude(mut paths: Vec<Value>, previous: Option<&Value>) -> Value {
    if let Some(previous) = previous {
        paths.push(previous.clone());
    }
    let first = paths[0].clone();
    if paths
        .iter()
        .all(|value| same_kind(&value.kind, &first.kind))
    {
        return first;
    }
    Value {
        kind: Kind::OneOf(paths),
        file: first.file.clone(),
        line: first.line,
    }
}

pub(super) fn same_kind(left: &Kind, right: &Kind) -> bool {
    match (left, right) {
        (Kind::Literal(left), Kind::Literal(right)) => left == right,
        (Kind::Unknown, Kind::Unknown) => true,
        (Kind::Array(left), Kind::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| same_kind(&left.kind, &right.kind))
        }
        (Kind::Object(left, left_opaque), Kind::Object(right, right_opaque)) => {
            left_opaque == right_opaque
                && left.len() == right.len()
                && left.iter().zip(right).all(
                    |((left_key, left_value), (right_key, right_value))| {
                        left_key == right_key && same_kind(&left_value.kind, &right_value.kind)
                    },
                )
        }
        (Kind::OneOf(left), Kind::OneOf(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| same_kind(&left.kind, &right.kind))
        }
        _ => false,
    }
}
