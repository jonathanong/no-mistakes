use super::*;
use crate::codebase::ts_source::facts::ancestry::{ClassShape, NamespaceValue};

#[derive(Clone, Copy)]
pub(super) enum Binding {
    Absent,
    Class(CallableId),
    Opaque,
}

pub(super) fn binding(file: &TsFileFacts, class: &ClassShape, name: &str) -> Binding {
    let (head, tail) = name.split_once('.').unwrap_or((name, ""));
    for depth in (1..=class.namespace.len()).rev() {
        let namespace = &class.namespace[..depth];
        // Private declarations belong to this actual lexical body, not every
        // declaration block that happens to have the same namespace path.
        let values = file
            .ancestry
            .namespace_bodies
            .iter()
            .filter(|body| body.namespace == namespace && class.bodies.contains(&body.id))
            .flat_map(|body| &body.values)
            .filter(|value| value.name == head)
            .collect::<Vec<_>>();
        if !values.is_empty() {
            return classify(&values, tail);
        }
        let shared = file
            .ancestry
            .namespace_bodies
            .iter()
            .filter(|body| body.namespace == namespace)
            .flat_map(|body| &body.values)
            .filter(|value| value.exported && value.name == head)
            .collect::<Vec<_>>();
        if !shared.is_empty() {
            return classify(&shared, tail);
        }
    }
    Binding::Absent
}

fn classify(values: &[&NamespaceValue], tail: &str) -> Binding {
    if !tail.is_empty() {
        return Binding::Opaque;
    }
    let Some(id) = values[0].class else {
        return Binding::Opaque;
    };
    if values.iter().all(|value| value.class == Some(id)) {
        Binding::Class(id)
    } else {
        Binding::Opaque
    }
}
