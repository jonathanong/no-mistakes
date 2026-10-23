mod api;
mod callee;
mod constants;
mod literals;
mod normalize;
mod regex_sample;
mod statics;
mod visitor;

#[cfg(test)]
mod tests;

pub use api::{
    extract_playwright_url_literals_from_path, extract_playwright_url_literals_from_program,
    extract_playwright_url_literals_with_helpers, extract_playwright_url_occurrences,
    extract_playwright_url_occurrences_from_program, extract_playwright_urls,
};

pub(crate) use constants::static_templates_from_semantic;
pub(crate) use visitor::extract_with_static_templates;
