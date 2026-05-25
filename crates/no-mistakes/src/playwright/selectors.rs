mod call_shapes;
mod code_only_text;
mod css;
mod extract_app;
mod extract_playwright;
mod matcher;
mod regex_mod;
pub(crate) mod scoped_defaults;
mod shadowing;
mod template;
#[cfg(test)]
mod tests;
mod text_locator_calls;
mod text_locators;
mod types;

pub use extract_app::extract_app_selectors_with_regexes;
pub use extract_app::{collect_app_selectors, extract_app_selectors};
pub use extract_playwright::extract_playwright_selector_occurrences_from_program;
pub use regex_mod::compile_selector_regexes;
pub use regex_mod::compile_selector_regexes_with_html_ids;
pub(crate) use text_locators::extract_playwright_text_locator_occurrences_from_program;
pub(crate) use types::{AppSelector, AppSelectorValue, PlaywrightSelector, SelectorRegexes};
pub use types::{SelectorMatcher, TemplatePattern};

pub(crate) const HTML_ID_ATTRIBUTE: &str = "id";

const SOURCE_EXTS: &[&str] = &["ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"];

pub fn is_source_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| SOURCE_EXTS.contains(&extension))
}
pub(crate) fn is_skipped_dir(path: &std::path::Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| matches!(name, ".git" | "node_modules" | "target" | "dist" | "build"))
}
pub fn extract_playwright_selectors(
    source: &str,
    selector_attributes: &[String],
    test_id_attributes: &[String],
) -> Vec<PlaywrightSelector> {
    use crate::playwright::ast;
    use std::path::Path;
    let regexes = regex_mod::compile_selector_regexes(
        selector_attributes,
        &std::collections::BTreeMap::new(),
    );
    ast::with_program(Path::new("fixture.ts"), source, |program, source| {
        extract_playwright::extract_playwright_selector_occurrences_from_program(
            program,
            source,
            &regexes,
            test_id_attributes,
        )
        .into_iter()
        .map(|o| o.value)
        .collect()
    })
    .expect("fixture should parse")
}
