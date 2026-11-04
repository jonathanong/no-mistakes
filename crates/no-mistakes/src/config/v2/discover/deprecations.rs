use super::{NoMistakesConfig, Path};

pub(super) fn emit_v2_deprecation_warnings(config: &NoMistakesConfig, path: &Path) {
    if config.test_plan.playwright.deprecated_dependencies_key {
        let path_display = declaration_path(config, path, "playwright").display();
        eprintln!(
            "warning: {path_display}: `test_plan.playwright.dependencies` is deprecated; \
             rename it to `test_plan.playwright.fullSuiteTriggers`"
        );
    }
    if config.test_plan.vitest.deprecated_dependencies_key {
        let path_display = declaration_path(config, path, "vitest").display();
        eprintln!(
            "warning: {path_display}: `test_plan.vitest.dependencies` is deprecated; \
             rename it to `test_plan.vitest.fullSuiteTriggers`"
        );
    }
}

fn declaration_path<'a>(
    config: &'a NoMistakesConfig,
    fallback: &'a Path,
    framework: &str,
) -> &'a Path {
    let key = crate::config::source_locations::ConfigPath::default()
        .key("testPlan")
        .key(framework)
        .key("dependencies");
    config
        .config_provenance
        .origin(&key)
        .map_or(fallback, |(path, _)| path)
}
