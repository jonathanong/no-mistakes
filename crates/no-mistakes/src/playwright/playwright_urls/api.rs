pub use super::visitor::extract_playwright_url_occurrences_from_program;

use crate::playwright::{ast, playwright_tests};
use oxc_ast::ast::Program;
use {anyhow::Result, std::path::Path};
pub fn extract_playwright_urls(source: &str) -> Vec<String> {
    extract_playwright_url_literals_with_helpers(source, &[])
        .into_iter()
        .filter(|url| url.starts_with('/'))
        .collect()
}
pub fn extract_playwright_url_literals_with_helpers(
    source: &str,
    navigation_helpers: &[String],
) -> Vec<String> {
    extract_playwright_url_literals_from_path(Path::new("fixture.ts"), source, navigation_helpers)
        .expect("fixture should parse")
}
pub fn extract_playwright_url_literals_from_path(
    path: &Path,
    source: &str,
    navigation_helpers: &[String],
) -> Result<Vec<String>> {
    ast::with_program(path, source, |program, source| {
        extract_playwright_url_literals_from_program(program, source, navigation_helpers)
    })
}
pub fn extract_playwright_url_occurrences(
    source: &str,
) -> Vec<(String, playwright_tests::TestStatus)> {
    ast::with_program(Path::new("fixture.ts"), source, |program, source| {
        let mut occurrences = Vec::new();
        for occurrence in extract_playwright_url_occurrences_from_program(program, source, &[]) {
            let value = (occurrence.value, occurrence.status);
            if !occurrences.contains(&value) {
                occurrences.push(value);
            }
        }
        occurrences
    })
    .expect("fixture should parse")
}
pub fn extract_playwright_url_literals_from_program(
    program: &Program<'_>,
    source: &str,
    navigation_helpers: &[String],
) -> Vec<String> {
    let mut urls: Vec<_> =
        extract_playwright_url_occurrences_from_program(program, source, navigation_helpers)
            .into_iter()
            .map(|occurrence| occurrence.value)
            .collect();
    urls.sort();
    urls.dedup();
    urls
}
