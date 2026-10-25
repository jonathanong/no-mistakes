use super::*;

pub(super) fn bindings(
    at: Span,
    facts: &FxHashMap<Span, Taint>,
    index: &Index,
    writes: &FxHashSet<usize>,
    values: &mut FxHashMap<usize, Taint>,
    queue: &mut VecDeque<Span>,
) {
    for (id, path) in index.outputs.get(&at).into_iter().flatten() {
        let mut value = facts[&at].clone();
        for key in path {
            value = value.read(key.as_deref());
        }
        if value.derived && writes.contains(id) {
            value.broad = true;
            value.paths.clear();
        }
        if values.entry(*id).or_default().join(value) {
            queue.extend(index.bindings.get(id).into_iter().flatten().copied());
        }
    }
    for id in index.stores.get(&at).into_iter().flatten() {
        let mut value = facts[&at].clone();
        if value.present() {
            value.broad = true;
            value.paths.clear();
        }
        if values.entry(*id).or_default().join(value) {
            queue.extend(index.bindings.get(id).into_iter().flatten().copied());
        }
    }
}
