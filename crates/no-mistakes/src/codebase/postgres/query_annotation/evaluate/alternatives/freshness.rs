use super::super::Environment;
use crate::codebase::postgres::query_annotation::evaluate::Scope;
use crate::fx::{FxHashMap, FxHashSet};

type Proofs = FxHashMap<Environment, FxHashSet<String>>;

pub(super) fn restore(proofs: &mut Proofs, count: usize, original: &Proofs) {
    proofs.retain(|env, _| *env >= count);
    proofs.extend(original.clone());
}

pub(super) fn unchanged(proofs: &mut Proofs, current: &Proofs, before: &[Scope], after: &[Scope]) {
    for (env, names) in proofs {
        let fresh = current.get(env);
        // Empty/nonfresh sets never indexed the scope slices. Keep that proof
        // lazy, while reusing it for every eligible name in this environment.
        let mut shared = None;
        names.retain(|name| {
            fresh.is_some_and(|fresh| fresh.contains(name))
                && (*shared.get_or_insert_with(|| before[*env].same_bindings(&after[*env])) || {
                    #[cfg(test)]
                    tests::record_binding_reads();
                    before[*env].get(name) == after[*env].get(name)
                })
        });
    }
}

pub(super) fn remap(
    proofs: &mut Proofs,
    count: usize,
    indices: &FxHashMap<Environment, Environment>,
) {
    *proofs = std::mem::take(proofs)
        .into_iter()
        .filter_map(|(env, names)| {
            if env < count {
                Some((env, names))
            } else {
                indices.get(&env).map(|new| (*new, names))
            }
        })
        .collect();
}

#[cfg(test)]
mod tests;
