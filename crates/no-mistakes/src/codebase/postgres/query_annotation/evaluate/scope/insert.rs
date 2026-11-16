use super::*;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Projection {
    Environment(usize),
    Argument(u64),
    Builder(u64),
    Container(*const ()),
    Empty,
}

pub(super) fn projection(mut value: &Value) -> Projection {
    loop {
        match value {
            Value::Function(_, _, env) => return Projection::Environment(*env),
            Value::Arguments(id) | Value::Array(id) => return Projection::Argument(*id),
            Value::Prefix(_, _, Some(id)) => return Projection::Builder(*id),
            Value::Aggregate(values)
            | Value::References(values)
            | Value::Object(values)
            | Value::Record(values, _)
            | Value::Joined(values)
            | Value::Possible(values) => return Projection::Container(values.identity()),
            Value::Promise(inner) | Value::Evaluated(inner, _) => value = inner,
            _ => return Projection::Empty,
        }
    }
}

impl Scope {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn insert(
        &mut self,
        name: String,
        value: Value,
    ) -> Option<Value> {
        let occupied = self.0.values.contains_key(&name);
        let (same_membership, same_edges) =
            self.0.values.get(&name).map_or((false, false), |before| {
                (
                    before.contains_reference() == value.contains_reference(),
                    projection(before) == projection(&value),
                )
            });
        let bindings = Arc::make_mut(&mut self.0);
        if !same_membership {
            if occupied {
                bindings.contains_reference.take();
            }
            bindings.reference_binding_names.take();
        }
        if !occupied {
            super::extend::new_binding(bindings, &value);
        } else if !same_edges {
            bindings.builders.take();
            bindings.references.take();
        }
        bindings.values.insert(name, value)
    }
}

#[cfg(test)]
mod tests;
