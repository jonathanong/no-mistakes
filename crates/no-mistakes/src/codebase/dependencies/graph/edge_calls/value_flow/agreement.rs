use super::*;

#[derive(Default)]
struct Provenance {
    typed: Vec<(String, String)>,
    members: Vec<(String, String, String)>,
    factories: Vec<(String, String)>,
    annotations: Vec<(std::path::PathBuf, std::sync::Arc<crate::codebase::dependencies::extract::ValueTypeSource>)>,
}

pub(super) fn agreed_value(values: Vec<FlowValue>) -> FlowValue {
    let mut values = values.into_iter().map(|value| {
        let mut provenance = Provenance::default();
        let value = normalize(value, &mut provenance);
        (value, provenance)
    });
    // Parameters supply a default candidate; indexed return lists are non-empty.
    let (first, mut shared) = values.next().expect("agreement requires a candidate value");
    for (value, provenance) in values {
        if value != first {
            return FlowValue::Unknown;
        }
        shared.annotations.retain(|origin| provenance.annotations.contains(origin));
        shared
            .typed
            .retain(|origin| provenance.typed.contains(origin));
        shared
            .members
            .retain(|origin| provenance.members.contains(origin));
        shared
            .factories
            .retain(|origin| provenance.factories.contains(origin));
    }
    let mut result = if shared.typed.is_empty() && shared.members.is_empty() && shared.factories.is_empty() {
        first
    } else {
        FlowValue::Provenance {
            executor: Box::new(first), typed: shared.typed, members: shared.members,
            receiver_factories: shared.factories,
        }
    };
    for (file, source) in shared.annotations {
        result = FlowValue::Annotated { executor: Box::new(result), file, source };
    }
    result
}

// New annotation/member proofs do not participate in ordinary value agreement.
// Only origins common to every input can select a configured effect afterward.
fn normalize(value: FlowValue, provenance: &mut Provenance) -> FlowValue {
    match value {
        FlowValue::Annotated { executor, file, source } => {
            provenance.annotations.push((file, source));
            normalize(*executor, provenance)
        },
        FlowValue::Provenance {
            executor,
            typed,
            members,
            receiver_factories,
        } => {
            provenance.typed.extend(typed);
            provenance.members.extend(members);
            provenance.factories.extend(receiver_factories);
            normalize(*executor, provenance)
        }
        FlowValue::Bound { target, arguments } => FlowValue::Bound {
            target: Box::new(normalize(*target, provenance)),
            arguments,
        },
        FlowValue::Promise(inner) => FlowValue::Promise(Box::new(normalize(*inner, provenance))),
        FlowValue::Returned {
            factory,
            executor,
            receiver,
        } => FlowValue::Returned {
            factory,
            executor: Box::new(normalize(*executor, provenance)),
            receiver,
        },
        other => other,
    }
}
