use super::CheckFactPlan;
use std::{path::PathBuf, sync::Arc};

impl CheckFactPlan {
    /// Add exact call-source demand without widening generic graph consumers.
    #[doc(hidden)]
    pub fn require_call_site_paths(&mut self, paths: &[PathBuf]) {
        if !paths.is_empty() {
            Arc::make_mut(&mut self.call_site_paths).extend(
                paths
                    .iter()
                    .map(|path| crate::codebase::ts_resolver::normalize_path(path)),
            );
        }
    }

    pub(super) fn merge_call_site_paths(&mut self, other: &Self) {
        if other.call_site_paths.is_empty()
            || Arc::ptr_eq(&self.call_site_paths, &other.call_site_paths)
        {
            return;
        }
        if self.call_site_paths.is_empty() {
            self.call_site_paths = Arc::clone(&other.call_site_paths);
        } else {
            Arc::make_mut(&mut self.call_site_paths).extend(other.call_site_paths.iter().cloned());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_site_demand_unions_without_copying_unchanged_shared_paths() {
        let mut plan = CheckFactPlan::default();
        plan.require_call_site_paths(&[PathBuf::from("/fixture/src/../entry.ts")]);
        let shared = Arc::clone(&plan.call_site_paths);
        plan.include(plan.clone());
        plan.include(CheckFactPlan::default());
        assert!(Arc::ptr_eq(&shared, &plan.call_site_paths));
        assert!(plan
            .call_site_paths
            .contains(&PathBuf::from("/fixture/entry.ts")));
        let mut empty = CheckFactPlan::default();
        empty.include(plan.clone());
        assert!(Arc::ptr_eq(&shared, &empty.call_site_paths));
        let mut other = CheckFactPlan::default();
        other.require_call_site_paths(&[PathBuf::from("/fixture/helper.ts")]);
        plan.include(other);
        assert_eq!(plan.call_site_paths.len(), 2);
        assert_eq!(shared.len(), 1, "union must not widen another request");
        assert!(
            !plan.graph.call_sites,
            "scoped demand never widens broad demand"
        );
    }
}
