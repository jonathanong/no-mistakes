use super::{PlaywrightFactPlan, PlaywrightFileFactPlan, PlaywrightSourceFactPlan, VariantPlan};
use std::collections::BTreeSet;
use std::sync::Arc;

impl PlaywrightFactPlan {
    #[doc(hidden)]
    pub fn include(&mut self, other: Self) {
        for (path, other_file) in other.files {
            let file = self
                .files
                .entry(path)
                .or_insert_with(PlaywrightFileFactPlan::empty);
            for (key, other_variant) in other_file.variants {
                let variant = file.variants.entry(key).or_insert_with(|| VariantPlan {
                    selector_regexes: Arc::clone(&other_variant.selector_regexes),
                    policies: Vec::new(),
                });
                merge_sorted(&mut variant.policies, other_variant.policies);
            }
        }

        let mut source_files = self.source_files.as_ref().clone();
        source_files.extend(other.source_files.iter().cloned());
        self.set_source_files(source_files);
        for source_plan in other.source_plans {
            self.merge_source_plan(source_plan);
        }
        Arc::make_mut(&mut self.config_files).extend(other.config_files.iter().cloned());

        let mut test_files = self.test_files_by_project.as_ref().clone();
        for (project, other_files) in other.test_files_by_project.iter() {
            match test_files
                .iter_mut()
                .find(|(candidate, _)| candidate == project)
            {
                Some((_, files)) => {
                    let mut merged = files.as_ref().clone();
                    merged.extend(other_files.iter().cloned());
                    merged.sort_by(|left, right| left.path.cmp(&right.path));
                    merged.dedup_by(|left, right| left.path == right.path);
                    *files = Arc::new(merged);
                }
                None => test_files.push((project.clone(), Arc::clone(other_files))),
            }
        }
        self.set_test_files_by_project(test_files);
    }

    pub(super) fn merge_source_plan(&mut self, source_plan: PlaywrightSourceFactPlan) {
        let Some(existing) = self
            .source_plans
            .iter_mut()
            .find(|existing| existing.settings == source_plan.settings)
        else {
            self.source_plans.push(source_plan);
            return;
        };
        Arc::make_mut(&mut existing.app_source_files)
            .extend(source_plan.app_source_files.iter().cloned());
        Arc::make_mut(&mut existing.visible_files)
            .extend(source_plan.visible_files.iter().cloned());
        if source_plan.scan_html_ids && !existing.scan_html_ids {
            existing.scan_html_ids = true;
            existing.selector_regexes = Arc::new(
                crate::playwright::selectors::compile_selector_regexes_with_html_ids(
                    &existing.settings.selector_attributes,
                    &existing.settings.component_selector_attributes,
                    true,
                ),
            );
        }
    }
}

pub(super) fn merge_sorted<T: Ord + Clone>(
    values: &mut Vec<T>,
    additions: impl IntoIterator<Item = T>,
) {
    let mut merged: BTreeSet<T> = values.iter().cloned().collect();
    merged.extend(additions);
    *values = merged.into_iter().collect();
}
