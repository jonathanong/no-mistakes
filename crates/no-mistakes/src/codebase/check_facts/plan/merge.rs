use std::sync::Arc;

pub(super) fn merge_workspace_importers(
    existing: &mut Option<Arc<crate::fx::PathSet>>,
    incoming: Option<Arc<crate::fx::PathSet>>,
    already_requested: bool,
) {
    if !already_requested {
        *existing = incoming;
    } else if let (Some(current), Some(additional)) = (existing.as_mut(), incoming.as_ref()) {
        if !additional.is_empty() && !Arc::ptr_eq(current, additional) {
            Arc::make_mut(current).extend(additional.iter().cloned());
        }
    } else {
        *existing = None;
    }
}
