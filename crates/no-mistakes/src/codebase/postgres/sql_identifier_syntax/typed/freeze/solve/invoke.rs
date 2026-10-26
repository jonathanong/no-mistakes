use super::*;

pub(super) fn returned(
    callee: Taint,
    mut value: Taint,
    functions: &FxHashMap<usize, Taint>,
    record_invoke: &mut impl FnMut(usize),
) -> Taint {
    // Producer identities carry their existing return summaries through projections.
    for (path, id) in &callee.producers {
        if path.is_empty() {
            record_invoke(*id);
            value.join(functions.get(id).cloned().unwrap_or_default());
        }
    }
    if callee.widened_factory {
        // An uncertain factory returns a carrier, not another implicit factory.
        value.join(Taint {
            broad: true,
            derived: true,
            ..Taint::default()
        });
    }
    value.derived |= callee.derived;
    value
}
