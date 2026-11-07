use super::candidates::Candidate;
use super::index::Index;
use super::*;

pub(super) fn bindings(
    index: &Index<'_>,
    candidate: &Candidate<'_>,
) -> Option<Vec<BoundedBinding>> {
    if !candidate.valid {
        return None;
    }
    let mut bounded = Vec::new();
    let Some(calls) = index.calls.get(&candidate.owner) else {
        return Some(bounded);
    };
    for call in calls {
        if call.bound || call.spread_at.is_some() {
            return None;
        }
        if let Some(results) = index.results.get(&(call.offset, call.end)) {
            let [binding] = results.as_slice() else {
                return None;
            };
            if !binding.immutable
                || binding.invalidated_at.is_some()
                || index
                    .mutated
                    .contains(&(binding.scope, binding.name.as_str()))
                || !index.reads((binding.scope, &binding.name), &candidate.fields)
            {
                return None;
            }
            for field in &candidate.fields {
                bounded.push(BoundedBinding {
                    name: binding.name.clone(),
                    scope: binding.scope,
                    declared_at: binding.declared_at,
                    source: Some(candidate.sources[0].clone()),
                    members: vec![field.to_string()],
                    requires_guard: false,
                });
            }
        } else if !index
            .collector
            .metadata_rows
            .direct
            .get(&call.offset)
            .is_some_and(|usage| {
                !usage.invalid
                    && !usage.fields.is_empty()
                    && usage
                        .fields
                        .iter()
                        .all(|field| candidate.fields.contains(field.as_str()))
            })
        {
            return None;
        }
    }
    bounded.sort_by(|left, right| {
        (&left.name, left.scope, left.declared_at, &left.members).cmp(&(
            &right.name,
            right.scope,
            right.declared_at,
            &right.members,
        ))
    });
    Some(bounded)
}
