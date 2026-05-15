pub use super::visitor::extract_playwright_url_occurrences_from_program;

#[cfg(test)]
use crate::{ast, playwright_tests};
#[cfg(test)]
use oxc_ast::ast::Program;
#[cfg(test)]
use {anyhow::Result, std::path::Path};

#[cfg(test)]
pub fn extract_playwright_urls(source: &str) -> Vec<String> {
    extract_playwright_url_literals_with_helpers(source, &[])
        .into_iter()
        .filter(|url| url.starts_with('/'))
        .collect()
}

#[cfg(test)]
pub fn extract_playwright_url_literals_with_helpers(
    source: &str,
    navigation_helpers: &[String],
) -> Vec<String> {
    extract_playwright_url_literals_from_path(Path::new("fixture.ts"), source, navigation_helpers)
        .expect("fixture should parse")
}

#[cfg(test)]
pub fn extract_playwright_url_literals_from_path(
    path: &Path,
    source: &str,
    navigation_helpers: &[String],
) -> Result<Vec<String>> {
    ast::with_program(path, source, |program, source| {
        extract_playwright_url_literals_from_program(program, source, navigation_helpers)
    })
}

#[cfg(test)]
pub fn extract_playwright_url_occurrences(
    source: &str,
) -> Vec<(String, playwright_tests::TestStatus)> {
    ast::with_program(Path::new("fixture.ts"), source, |program, source| {
        extract_playwright_url_occurrences_from_program(program, source, &[])
            .into_iter()
            .map(|occurrence| (occurrence.value, occurrence.status))
            .collect()
    })
    .expect("fixture should parse")
}

#[cfg(test)]
pub fn extract_playwright_url_literals_from_program(
    program: &Program<'_>,
    source: &str,
    navigation_helpers: &[String],
) -> Vec<String> {
    extract_playwright_url_occurrences_from_program(program, source, navigation_helpers)
        .into_iter()
        .map(|occurrence| occurrence.value)
        .collect()
}
