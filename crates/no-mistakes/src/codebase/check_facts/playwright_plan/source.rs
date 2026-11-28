use super::{PlaywrightFactPlan, PlaywrightSettingsKey, PlaywrightSourceFactPlan};
use crate::codebase::playwright_facts::{
    paths::{build_globset, relative_string},
    DiscoveredTestFile, Settings,
};
use crate::codebase::playwright_hooks;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl PlaywrightFactPlan {
    pub fn set_source_files(&mut self, files: impl AsRef<Vec<PathBuf>>) {
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

    pub fn add_source_settings(
        &mut self,
        root: &Path,
        settings: Settings,
        scan_html_ids: bool,
        snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
    ) -> anyhow::Result<()> {
        crate::codebase::playwright_facts::integration::validate(&settings.route_coverage_sources)?;
        let sources = snapshot.source_store_for(root);
        let visible_files = sources
            .inventory()
            .paths()
            .iter()
            .cloned()
            .collect::<crate::fx::PathSet>();
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

        let include = build_globset(&settings.selector_include)?;
        let exclude = build_globset(&settings.selector_exclude)?;
        let app_source_files = (playwright_hooks::hooks().selector_source_files)(
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
        let selector_regexes = self.regex_cache.get_or_compile(
            &settings.selector_attributes,
            &settings.component_selector_attributes,
            settings.html_ids || scan_html_ids,
        );
        let settings_key = PlaywrightSettingsKey::new(&settings);
        let integration_sources = settings
            .route_coverage_sources
            .iter()
            .map(|source| Ok((source.clone(), build_globset(&source.include)?)))
            .collect::<anyhow::Result<Vec<_>>>()?;
        self.merge_source_plan(PlaywrightSourceFactPlan {
            app_source_files: Arc::new(app_source_files),
            selector_regexes,
            settings: Arc::new(settings),
            visible_files: Arc::new(visible_files),
            scan_html_ids,
            settings_key,
            integration_sources,
        });
        Ok(())
    }

    pub fn require_html_id_scan(&mut self, settings: &Settings) {
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

    pub fn integration_route_settings(&self) -> impl Iterator<Item = &Settings> {
        self.source_plans
            .iter()
            .map(|plan| plan.settings.as_ref())
            .filter(|settings| !settings.route_coverage_sources.is_empty())
    }

    pub fn integration_route_sources(
        &self,
        root: &Path,
        path: &Path,
    ) -> impl Iterator<Item = &crate::config::v2::schema::RouteCoverageSource> {
        let normalized_root = crate::codebase::ts_resolver::normalize_path(root);
        let relative = relative_string(&normalized_root, path);
        self.source_plans
            .iter()
            .flat_map(|plan| plan.integration_sources.iter())
            .filter(move |(_, include)| include.is_match(&relative))
            .map(|(source, _)| source)
    }

    pub(crate) fn set_test_files_by_project(
        &mut self,
        mut files: Vec<(Option<String>, Arc<Vec<DiscoveredTestFile>>)>,
    ) {
        files.sort_by(|left, right| left.0.cmp(&right.0));
        files.dedup_by(|left, right| left.0 == right.0);
        self.test_files_by_project = Arc::new(files);
    }

    pub(crate) fn add_test_files_for_project(
        &mut self,
        project: Option<String>,
        files: Arc<Vec<DiscoveredTestFile>>,
    ) {
        let mut test_files = self.test_files_by_project.as_ref().clone();
        match test_files
            .iter_mut()
            .find(|(candidate, _)| candidate == &project)
        {
            Some((_, existing)) => {
                let mut merged = existing.as_ref().clone();
                merged.extend(files.iter().cloned());
                merged.sort_by(|left, right| left.path.cmp(&right.path));
                merged.dedup_by(|left, right| left.path == right.path);
                *existing = Arc::new(merged);
            }
            None => test_files.push((project, files)),
        }
        self.set_test_files_by_project(test_files);
    }

    pub fn source_files(&self) -> Arc<Vec<PathBuf>> {
        Arc::clone(&self.source_files)
    }

    pub fn files(&self) -> &[PathBuf] {
        self.source_files.as_slice()
    }

    pub(crate) fn test_files_by_project(&self) -> super::super::PlaywrightTestFilesByProject {
        Arc::clone(&self.test_files_by_project)
    }

    pub fn contains_source(&self, path: &Path) -> bool {
        self.source_file_set.contains(path)
    }

    pub fn source_file_set(&self) -> &crate::fx::PathSet {
        &self.source_file_set
    }

    pub fn config_files(&self) -> &crate::fx::PathSet {
        &self.config_files
    }

    pub fn source_plans_for<'a>(
        &'a self,
        path: &'a Path,
    ) -> impl Iterator<Item = &'a PlaywrightSourceFactPlan> + 'a {
        self.source_plans
            .iter()
            .filter(move |plan| plan.app_source_files.contains(path))
    }
}
