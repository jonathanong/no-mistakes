use super::{NoMistakesConfig, Path};

pub(super) fn emit_v2_deprecation_warnings(config: &NoMistakesConfig, path: &Path) {
    let path_display = path.display();
    if config.test_plan.playwright.deprecated_dependencies_key {
        eprintln!(
            "warning: {path_display}: `test_plan.playwright.dependencies` is deprecated; \
             rename it to `test_plan.playwright.fullSuiteTriggers`"
        );
    }
    if config.test_plan.vitest.deprecated_dependencies_key {
        eprintln!(
            "warning: {path_display}: `test_plan.vitest.dependencies` is deprecated; \
             rename it to `test_plan.vitest.fullSuiteTriggers`"
        );
    }
}
