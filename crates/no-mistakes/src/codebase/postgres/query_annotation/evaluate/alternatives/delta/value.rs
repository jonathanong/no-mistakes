use super::super::super::{
    transactions::{Cell, Change, Saved},
    Value,
};
use crate::fx::{FxHashMap, FxHashSet};
fn prefixes(value: &Value, out: &mut FxHashMap<u64, Value>) {
    match value {
        value @ Value::Prefix(_, _, Some(id)) => {
            out.insert(*id, value.clone());
        }
        Value::Promise(value) | Value::Evaluated(value, _) => prefixes(value, out),
        Value::Aggregate(values)
        | Value::References(values)
        | Value::Object(values)
        | Value::Record(values, _)
        | Value::Joined(values)
        | Value::Possible(values)
            if values.contains_reference() =>
        {
            for value in values {
                prefixes(value, out);
            }
        }
        _ => {}
    }
}
pub(super) fn builders(changes: &[Change]) -> FxHashMap<u64, Value> {
    let mut out = FxHashMap::default();
    for change in changes {
        if matches!(
            change.cell,
            Cell::Binding(..) | Cell::Dense(..) | Cell::Extra(..)
        ) {
            if let Saved::Value(Some(value)) = &change.before {
                prefixes(value, &mut out);
            }
        }
    }
    out
}
pub(super) fn changes(change: &Change, changed: &mut FxHashSet<u64>) {
    if !matches!(
        change.cell,
        Cell::Binding(..) | Cell::Dense(..) | Cell::Extra(..)
    ) {
        return;
    }
    if let (Saved::Value(Some(before)), Saved::Value(Some(after))) = (&change.before, &change.after)
    {
        let (mut previous, mut current) = (FxHashMap::default(), FxHashMap::default());
        prefixes(before, &mut previous);
        prefixes(after, &mut current);
        for (id, value) in previous {
            if let (Value::Prefix(text, done, _), Some(Value::Prefix(other, complete, _))) =
                (value, current.get(&id))
            {
                if (text != *other || done != *complete)
                    && !(text.trim_start().starts_with("/*")
                        && other.trim_start().starts_with("/*"))
                {
                    changed.insert(id);
                }
            }
        }
    }
}
pub(super) fn join(
    cell: &Cell,
    initial: &Saved,
    before: &Saved,
    after: &Saved,
    whole_sql: bool,
    shapes: &mut FxHashSet<u64>,
) -> Saved {
    if before == after {
        return before.clone();
    }
    match (cell, initial, before, after) {
        (
            Cell::Binding(..) | Cell::Builder(_),
            Saved::Value(initial),
            Saved::Value(before),
            Saved::Value(after),
        ) => Saved::Value(Some(super::super::bindings::join_value(
            initial.as_ref(),
            before.as_ref().unwrap_or(&Value::Unknown),
            after.as_ref(),
            whole_sql,
        ))),
        (Cell::Dense(..) | Cell::Extra(..), _, Saved::Value(before), Saved::Value(after)) => {
            Saved::Value(Some(super::super::merge::value(
                before.clone().unwrap_or(Value::Unknown),
                after.clone().unwrap_or(Value::Unknown),
            )))
        }
        (Cell::Length(id), _, Saved::Length(_), Saved::Length(_)) => {
            shapes.insert(*id);
            before.clone()
        }
        (
            Cell::Definite(..) | Cell::Disconnected(..) | Cell::CallbackSeen(_),
            _,
            Saved::Flag(before),
            Saved::Flag(after),
        ) => Saved::Flag(*before && *after),
        (Cell::Fresh(..), Saved::Flag(initial), Saved::Flag(before), Saved::Flag(after)) => {
            Saved::Flag(*initial && *before && *after)
        }
        (
            Cell::Recreated(..)
            | Cell::CallbackRunning(_)
            | Cell::CallbackEnabled
            | Cell::ExecutionsEnabled,
            _,
            _,
            _,
        ) => initial.clone(),
        (_, _, Saved::Flag(before), Saved::Flag(after)) => Saved::Flag(*before || *after),
        _ => initial.clone(),
    }
}

pub(super) fn first(cell: &Cell, initial: &Saved, after: &Saved) -> Saved {
    match (cell, initial, after) {
        (
            Cell::Recreated(..)
            | Cell::CallbackRunning(_)
            | Cell::CallbackEnabled
            | Cell::ExecutionsEnabled,
            _,
            _,
        ) => initial.clone(),
        (Cell::Fresh(..), Saved::Flag(initial), Saved::Flag(after)) => {
            Saved::Flag(*initial && *after)
        }
        _ => after.clone(),
    }
}
