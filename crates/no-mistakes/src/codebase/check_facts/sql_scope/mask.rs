use super::{BTreeMap, FactDemand, PathBuf};
use std::sync::Arc;

pub(super) fn remaining_sql_demands(
    demands: &mut BTreeMap<PathBuf, Vec<FactDemand>>,
    request: usize,
) {
    let mut masks = crate::fx::fx_map();
    for variant in demands
        .values_mut()
        .flatten()
        .filter(|variant| variant.request == request)
    {
        let (_, masked) = masks.entry(Arc::as_ptr(&variant.plan)).or_insert_with(|| {
            let mut plan = (*variant.plan).clone();
            let original_ts = plan.collected_ts_plan();
            plan.graph.include(original_ts);
            plan.embedded_sql = false;
            plan.embedded_sql_options.clear();
            plan.embedded_sql_head_calls_only.clear();
            // Retain the original so its address cannot be recycled during this pass.
            (Arc::clone(&variant.plan), Arc::new(plan))
        });
        variant.plan = Arc::clone(masked);
    }
}
