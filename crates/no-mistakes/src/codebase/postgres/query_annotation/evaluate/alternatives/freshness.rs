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
        names.retain(|name| {
            current.get(env).is_some_and(|fresh| fresh.contains(name))
                && before[*env].get(name) == after[*env].get(name)
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
