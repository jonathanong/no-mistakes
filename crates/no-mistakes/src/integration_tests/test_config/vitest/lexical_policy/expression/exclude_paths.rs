use super::{Kind, Value};
use std::path::{Path, PathBuf};

pub(super) fn unknown_value(file: &Path, line: usize) -> Value {
    Value {
        kind: Kind::Unknown,
        file: file.to_path_buf(),
        line,
    }
}

pub(super) fn merged_exclude(
    present: Vec<Value>,
    object_count: usize,
    absent: bool,
    previous: Option<&Value>,
    incoming_opaque: bool,
) -> Value {
    let present: Vec<Value> = present
        .into_iter()
        .map(|value| bind_omitted(&value, previous, incoming_opaque))
        .collect();
    let first = &present[0];
    let omitted = absent || present.len() != object_count;
    if incoming_opaque && previous.is_none() && omitted {
        return unknown_value(&first.file, first.line);
    }
    let mut paths = present;
    if omitted {
        paths.push(match previous {
            Some(previous) => previous.clone(),
            None => omitted_marker(),
        });
    }
    collapse(paths)
}

fn bind_omitted(value: &Value, previous: Option<&Value>, incoming_opaque: bool) -> Value {
    match &value.kind {
        Kind::Omitted => match previous {
            Some(previous) => previous.clone(),
            None if incoming_opaque => unknown_value(&value.file, value.line),
            None => value.clone(),
        },
        Kind::OneOf(options) => collapse(
            options
                .iter()
                .map(|option| bind_omitted(option, previous, incoming_opaque))
                .collect(),
        ),
        _ => value.clone(),
    }
}

fn omitted_marker() -> Value {
    Value {
        kind: Kind::Omitted,
        file: PathBuf::new(),
        line: 1,
    }
}

fn collapse(paths: Vec<Value>) -> Value {
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
