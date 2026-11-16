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
    let Some(edges) = arena
        .objects
        .get(id)
        .and_then(|slots| slots.cached_references())
    else {
        return false;
    };
    edges.identities.is_empty()
        && edges.arguments.is_empty()
        && arena.extras.get(id).is_none_or(|slots| slots.is_empty())
}
