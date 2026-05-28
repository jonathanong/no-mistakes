mod call_shapes;
mod code_only_text;
mod css;
mod dynamic_values;
mod extract_app;
mod extract_playwright;
mod jsx_resolve;
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

/// Best-effort scan of a sequence of source lines for occurrences of any
/// attribute in `attributes` set to a static string literal, e.g.
/// `data-pw="search-bar"`. Used to recover identifiers that have been removed
/// in a unified diff hunk where we no longer have the original AST. Skips
/// values that look dynamic (containing `{` or `$`).
pub fn scan_selector_attribute_values(
    attributes: &[String],
    lines: &[String],
) -> Vec<(String, String)> {
    if attributes.is_empty() || lines.is_empty() {
        return Vec::new();
    }
    let escaped: Vec<String> = attributes.iter().map(|a| regex::escape(a)).collect();
    let pattern = format!(
        r#"(?P<attr>{})\s*=\s*(?:"(?P<dq>[^"{{}}$]*)"|'(?P<sq>[^'{{}}$]*)')"#,
        escaped.join("|")
    );
    let Ok(re) = regex::Regex::new(&pattern) else {
        return Vec::new();
    };
    let mut out: Vec<(String, String)> = Vec::new();
    for line in lines {
        for caps in re.captures_iter(line) {
            let attr = caps.name("attr").map(|m| m.as_str().to_string());
            let value = caps
                .name("dq")
                .or_else(|| caps.name("sq"))
                .map(|m| m.as_str().to_string());
            if let (Some(a), Some(v)) = (attr, value) {
                if !v.is_empty() {
                    out.push((a, v));
                }
            }
        }
    }
    out
}

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
