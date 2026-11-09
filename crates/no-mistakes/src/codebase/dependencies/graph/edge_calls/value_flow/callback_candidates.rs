use super::*;

pub(super) fn select(values: Vec<FlowValue>) -> FlowValue {
    let mut known: Vec<_> = values.into_iter().filter_map(candidate).collect();
    match known.len() {
        0 => FlowValue::Unknown,
        1 => known.pop().unwrap(),
        _ => FlowValue::Choice(known),
    }
}
fn candidate(value: FlowValue) -> Option<FlowValue> {
    Some(match value {
        FlowValue::Callable { .. } => value,
        FlowValue::Target(ResolvedCallTarget::Unknown) => return None,
        FlowValue::Target(ResolvedCallTarget::ModuleExport {
            ref export_path, ..
        }) if export_path.is_empty() => return None,
        FlowValue::Target(ResolvedCallTarget::Alternatives(alternatives)) => {
            let mut known = (*alternatives).clone();
            known.known.retain(|candidate| {
                !matches!(&candidate.target,
                KnownCallTarget::ModuleExport { export_path, .. } if export_path.is_empty())
            });
            if known.known.is_empty() {
                return None;
            }
            known.has_unknown = false;
            FlowValue::Target(ResolvedCallTarget::Alternatives(std::sync::Arc::new(known)))
        }
        FlowValue::Target(_) => value,
        FlowValue::Choice(values) => {
            let known = select(values);
            if matches!(known, FlowValue::Unknown) {
                return None;
            }
            known
        }
        FlowValue::Possible(inner) => return candidate(*inner),
        FlowValue::Bound { target, arguments } => FlowValue::Bound {
            target: Box::new(candidate(*target)?),
            arguments,
        },
        FlowValue::Returned {
            factory,
            executor,
            receiver,
        } => FlowValue::Returned {
            factory,
            executor: Box::new(candidate(*executor)?),
            receiver,
        },
        FlowValue::TypedObject {
            executor,
            properties,
        } => FlowValue::TypedObject {
            executor: Box::new(candidate(*executor)?),
            properties,
        },
        FlowValue::Provenance {
            executor,
            typed,
            members,
            receiver_factories,
        } => FlowValue::Provenance {
            executor: Box::new(candidate(*executor)?),
            typed,
            members,
            receiver_factories,
        },
        _ => return None,
    })
}

#[cfg(test)]
#[path = "callback_candidates_tests.rs"]
mod tests;
