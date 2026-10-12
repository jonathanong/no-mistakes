use super::PreparedCompanionImportPlan;
use std::sync::Arc;

pub(crate) fn merge_plans(
    left: &Arc<Vec<Arc<PreparedCompanionImportPlan>>>,
    right: &Arc<Vec<Arc<PreparedCompanionImportPlan>>>,
) -> Arc<Vec<Arc<PreparedCompanionImportPlan>>> {
    if left.is_empty() {
        return Arc::clone(right);
    }
    if right.is_empty() || Arc::ptr_eq(left, right) {
        return Arc::clone(left);
    }
    let mut plans = left.as_ref().clone();
    for plan in right.iter() {
        if !plans.iter().any(|existing| Arc::ptr_eq(existing, plan)) {
            plans.push(Arc::clone(plan));
        }
    }
    Arc::new(plans)
}
