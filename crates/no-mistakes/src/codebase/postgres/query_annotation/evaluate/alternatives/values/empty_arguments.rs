use super::{Arena, Value};

pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn prefix_free_argument(
    mut value: &Value,
    arena: Arena<'_>,
) -> bool {
    let id = loop {
        match value {
            Value::Promise(inner) | Value::Evaluated(inner, _) => value = inner,
            Value::Arguments(id) | Value::Array(id) => break id,
            _ => return false,
        }
    };
    arena.extras.get(id).is_none_or(|slots| slots.is_empty())
        && arena
            .objects
            .get(id)
            .is_some_and(|slots| !slots.contains_reference())
}
