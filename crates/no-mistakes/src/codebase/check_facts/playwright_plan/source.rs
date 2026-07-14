use super::{PlaywrightFactPlan, PlaywrightSettingsKey, PlaywrightSourceFactPlan};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl PlaywrightFactPlan {
    pub(crate) fn set_source_files(&mut self, files: impl AsRef<Vec<PathBuf>>) {
        let mut files = files
            .as_ref()
            .iter()
            .map(|path| crate::codebase::ts_resolver::normalize_path(path))
            .collect::<Vec<_>>();
        files.sort();
        files.dedup();
        self.source_file_set = Arc::new(files.iter().cloned().collect());
        self.source_files = Arc::new(files);
    }

    pub(crate) fn add_source_settings(
        &mut self,
        root: &Path,
        settings: crate::playwright::config::Settings,
        scan_html_ids: bool,
        snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
    ) -> anyhow::Result<()> {
        let sources = snapshot.source_store_for(root);
        let visible_files = sources
            .inventory()
            .paths()
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        let playwright_configs = settings
            .playwright_configs
            .iter()
            .map(|path| crate::codebase::ts_resolver::normalize_path(path))
            .collect::<HashSet<_>>();
        Arc::make_mut(&mut self.config_files).extend(playwright_configs.iter().cloned());
        let source_files = visible_files
            .iter()
            .filter(|path| {
                crate::codebase::dependencies::extract::is_indexable(path)
                    && sources
                        .inventory()
                        .classification_for_path(path)
                        .is_some_and(crate::codebase::ts_source::FileClassification::target_is_file)
                    && !playwright_configs.contains(*path)
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut merged_source_files = self.source_files.as_ref().clone();
        merged_source_files.extend(source_files);
        self.set_source_files(merged_source_files);

        let include = crate::playwright::fsutil::build_globset(&settings.selector_include)?;
        let exclude = crate::playwright::fsutil::build_globset(&settings.selector_exclude)?;
        let app_source_files =
            crate::playwright::analysis::app_collect::collect_selector_source_files_from_visible(
                root,
                &settings,
                &include,
                &exclude,
                settings.selector_include.is_empty(),
                snapshot,
            )
            .into_iter()
            .map(|path| crate::codebase::ts_resolver::normalize_path(&path))
            .collect();
        let selector_regexes = Arc::new(
            crate::playwright::selectors::compile_selector_regexes_with_html_ids(
                &settings.selector_attributes,
                &settings.component_selector_attributes,
                settings.html_ids || scan_html_ids,
            ),
        );
        let settings_key = PlaywrightSettingsKey::new(&settings);
        self.merge_source_plan(PlaywrightSourceFactPlan {
            app_source_files: Arc::new(app_source_files),
            selector_regexes,
            settings: Arc::new(settings),
            visible_files: Arc::new(visible_files),
            scan_html_ids,
            settings_key,
        });
        Ok(())
    }

    pub(crate) fn require_html_id_scan(&mut self, settings: &crate::playwright::config::Settings) {
        let Some(mut source_plan) = self
            .source_plans
            .iter()
            .find(|source_plan| source_plan.settings_key == PlaywrightSettingsKey::new(settings))
            .cloned()
        else {
            return;
        };
        source_plan.scan_html_ids = true;
        self.merge_source_plan(source_plan);
    }

    pub(crate) fn set_test_files_by_project(
        &mut self,
        mut files: Vec<(
            Option<String>,
            Arc<Vec<crate::playwright::analysis::context::DiscoveredTestFile>>,
        )>,
    ) {
        files.sort_by(|left, right| left.0.cmp(&right.0));
        files.dedup_by(|left, right| left.0 == right.0);
        self.test_files_by_project = Arc::new(files);
    }

    pub(crate) fn source_files(&self) -> Arc<Vec<PathBuf>> {
        Arc::clone(&self.source_files)
    }

    pub(crate) fn files(&self) -> &[PathBuf] {
        self.source_files.as_slice()
    }

    pub(crate) fn test_files_by_project(&self) -> super::super::PlaywrightTestFilesByProject {
        Arc::clone(&self.test_files_by_project)
    }

    pub(crate) fn contains_source(&self, path: &Path) -> bool {
        self.source_file_set.contains(path)
    }

    pub(crate) fn source_file_set(&self) -> &HashSet<PathBuf> {
        &self.source_file_set
    }

    pub(crate) fn config_files(&self) -> &HashSet<PathBuf> {
        &self.config_files
    }

    pub(crate) fn source_plans_for<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl Iterator<Item = &'a PlaywrightSourceFactPlan> + 'a {
        self.source_plans
            .iter()
            .filter(move |plan| plan.app_source_files.contains(path))
    }
}
