use super::{PlaywrightSelectorVisitor, WrapperBindings};
use crate::playwright::playwright_tests;
use crate::playwright::selectors::types::{PlaywrightSelector, SelectorRegexes};
use oxc_ast_visit::Visit;
use std::collections::HashSet;

pub fn extract_playwright_selector_occurrences_from_program(
    program: &oxc_ast::ast::Program<'_>,
    source: &str,
    regexes: &SelectorRegexes,
    test_id_attributes: &[String],
    wrappers: &[crate::config::v2::schema::PlaywrightSelectorWrapper],
) -> Vec<playwright_tests::TestOccurrence<PlaywrightSelector>> {
    extract_playwright_selector_occurrences_and_wrapper_calls_from_program(
        program,
        source,
        regexes,
        test_id_attributes,
        wrappers,
        None,
        None,
    )
    .0
}

pub(crate) fn extract_playwright_selector_occurrences_and_wrapper_calls_from_program(
    program: &oxc_ast::ast::Program<'_>,
    source: &str,
    regexes: &SelectorRegexes,
    test_id_attributes: &[String],
    wrappers: &[crate::config::v2::schema::PlaywrightSelectorWrapper],
    importing_file: Option<&std::path::Path>,
    module_resolution: Option<&crate::codebase::check_facts::PlaywrightModuleResolution>,
) -> (
    Vec<playwright_tests::TestOccurrence<PlaywrightSelector>>,
    HashSet<u32>,
) {
    let wrapper_bindings =
        WrapperBindings::from_program(program, wrappers, importing_file, module_resolution);
    let mut visitor =
        PlaywrightSelectorVisitor::new(source, regexes, test_id_attributes, wrapper_bindings);
    visitor.visit_program(program);
    playwright_tests::dedup_occurrences_by_identity(&mut visitor.selectors);
    (visitor.selectors, visitor.wrapper_call_offsets)
}
