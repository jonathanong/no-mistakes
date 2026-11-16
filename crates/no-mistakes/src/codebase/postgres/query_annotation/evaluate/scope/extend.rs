use super::insert::{projection, Projection};
use super::*;

pub(super) fn new_binding(bindings: &mut Bindings, value: &Value) {
    if let Some(contains) = bindings.contains_reference.get_mut() {
        *contains |= value.contains_reference();
    }
    let projected = projection(value);
    if projected == Projection::Empty {
        return;
    }
    if let Projection::Container(_) = projected {
        bindings.references.take();
        bindings.builders.take();
        return;
    }
    if let Some(edges) = bindings.references.get_mut() {
        let edges = Arc::make_mut(edges);
        match projected {
            Projection::Environment(env) => {
                edges.environments.insert(env);
            }
            Projection::Argument(id) => {
                edges.arguments.insert(id);
            }
            Projection::Builder(id) => {
                edges.identities.insert(id);
            }
            Projection::Empty | Projection::Container(_) => {}
        }
    }
    if let Projection::Builder(id) = projected {
        if let Some(builders) = bindings.builders.get_mut() {
            Arc::make_mut(builders).insert(id);
        }
    }
}

#[cfg(test)]
mod tests;
