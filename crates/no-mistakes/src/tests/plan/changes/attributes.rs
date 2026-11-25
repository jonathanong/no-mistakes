use globset::{Glob, GlobSetBuilder};
use no_mistakes::changes::config::ChangeCategory;
use no_mistakes::changes::ChangesReport;
use no_mistakes::codebase::ts_source::{FixtureReadIndex, FixtureReadScope};
use no_mistakes::config::v2::NoMistakesConfig;
use std::{path::Path, sync::Arc};

pub(super) fn index(
    root: &Path,
    config: &NoMistakesConfig,
    report: &ChangesReport,
) -> Option<Arc<FixtureReadIndex>> {
    let scopes = report
        .categories
        .iter()
        .filter_map(|(name, changes)| {
            let category = &config.changes.categories[name];
            let template = loader_template(category)?;
            let literals = changes
                .changed
                .iter()
                .filter(|id| changes.attribute_paths.contains_key(*id))
                .map(|id| template.replace("{id}", id))
                .collect::<Vec<_>>();
            if literals.is_empty() {
                return None;
            }
            let mut tests = GlobSetBuilder::new();
            for glob in &category.tests {
                tests.add(Glob::new(glob.trim_start_matches("./")).ok()?);
            }
            FixtureReadScope::new(tests.build().ok()?, literals).ok()
        })
        .collect::<Vec<_>>();
    (!scopes.is_empty()).then(|| Arc::new(FixtureReadIndex::new(root.to_path_buf(), scopes)))
}

pub(super) fn loader_template(category: &ChangeCategory) -> Option<String> {
    if !category.loader_returns_parsed_json {
        return None;
    }
    let template = category.match_pattern.as_deref()?.replace("<id>", "{id}");
    (template.matches("{id}").count() == 1
        && (template.contains(r#""{id}""#) || template.contains("'{id}'")))
    .then_some(template)
}
