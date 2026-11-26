use super::index::{Identity, Index, Lookup};
use super::*;

pub(super) struct Candidate<'a> {
    pub(super) owner: CallableId,
    pub(super) sources: Vec<&'a (String, usize)>,
    pub(super) fields: FxHashSet<&'a str>,
    pub(super) valid: bool,
}

struct Returned<'a, 'i> {
    identity: Identity<'a>,
    lookup: &'i Lookup<'a>,
    offset: u32,
    owned: bool,
}

pub(super) fn collect<'a>(index: &Index<'a>, module: bool) -> Vec<Candidate<'a>> {
    index
        .returns
        .iter()
        .filter_map(|(owner, returns)| {
            let rows: Vec<_> = returns
                .iter()
                .map(|value| returned_row(index, *owner, value))
                .collect();
            let first = rows.iter().flatten().next()?;
            let mut fields = first.lookup.fields.clone();
            let mut sources = Vec::new();
            let mut valid = module && closed_helper(index, *owner);
            for row in rows {
                let Some(Returned {
                    identity,
                    lookup,
                    offset,
                    owned,
                }) = row
                else {
                    valid = false;
                    continue;
                };
                if !sources.contains(&lookup.source) {
                    sources.push(lookup.source);
                }
                fields.retain(|field| lookup.fields.contains(field));
                valid &= owned
                    && !index.mutated.contains(&(identity.0, identity.1))
                    && index.reads((identity.0, identity.1), &lookup.fields)
                    && index.guarded(identity, *owner, offset);
            }
            valid &= sources.len() == 1 && !fields.is_empty();
            Some(Candidate {
                owner: *owner,
                sources,
                fields,
                valid,
            })
        })
        .collect()
}

fn returned_row<'a, 'i>(
    index: &'i Index<'a>,
    owner: CallableId,
    value: &ValueExpression,
) -> Option<Returned<'a, 'i>> {
    let ValueExpression::Named {
        name,
        scope: Some(scope),
        offset,
        ..
    } = value
    else {
        return None;
    };
    let binding = index.binding((*scope, name), *offset)?;
    let identity = (binding.scope, binding.name.as_str(), binding.declared_at);
    let lookup = index.lookups.get(&identity)?;
    Some(Returned {
        identity,
        lookup,
        offset: *offset,
        owned: binding.immutable && binding.owner == Some(owner),
    })
}

fn closed_helper(index: &Index<'_>, owner: CallableId) -> bool {
    let collector = index.collector;
    if collector.value_flow.async_callables.contains(&owner)
        || collector.value_flow.generator_callables.contains(&owner)
    {
        return false;
    }
    let Some(names) = index.names.get(&owner) else {
        return false;
    };
    let [name] = names.as_slice() else {
        return false;
    };
    !collector.exported_functions.contains(*name)
        && !collector.has_reassigned_callable_at(0, name)
        && collector
            .value_flow
            .callable_value_uses
            .as_deref()
            .and_then(|uses| uses.get(&0))
            .is_none_or(|uses| !uses.contains(*name))
        && index
            .binding((0, name), u32::MAX)
            .is_none_or(|binding| binding.immutable)
}
