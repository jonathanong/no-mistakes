//! Extracted from `pipeline.rs`'s `analyze_with_policy_and_optional_facts`
//! purely to stay under the 200-code-line-per-file cap after adding
//! per-step timing there — no behavior change.

use super::context::{DiscoveredTestFile, TestAnalysisContext};
use super::test_file::{analyze_test_file, analyze_test_occurrences};
use super::types::TestFileAnalysis;
use anyhow::Result;
use rayon::prelude::*;

/// Analyze every discovered test file in parallel, reusing already-collected
/// Playwright facts per file when available (falling back to a full parse +
/// analysis for files the caller doesn't have facts for).
pub(crate) fn analyze_test_files(
    test_files: &[DiscoveredTestFile],
    test_analysis: &TestAnalysisContext<'_>,
    facts: Option<&dyn crate::codebase::dependencies::graph::TsFactLookup>,
) -> Result<TestFileAnalysis> {
    test_files
        .par_iter()
        .try_fold(
            TestFileAnalysis::default,
            |mut result, test_file| -> Result<_> {
                let file_analysis = if let Some(facts) = facts {
                    match facts.get_playwright_facts(&test_file.path) {
                        Some(playwright) => analyze_test_occurrences(
                            test_file,
                            test_analysis,
                            playwright.urls.clone(),
                            playwright.selectors.clone(),
                            playwright.text_locators.clone(),
                            playwright.helper_references.clone(),
                        ),
                        None => analyze_test_file(test_file, test_analysis)?,
                    }
                } else {
                    analyze_test_file(test_file, test_analysis)?
                };
                result.edges.extend(file_analysis.edges);
                result
                    .helper_references
                    .extend(file_analysis.helper_references);
                Ok(result)
            },
        )
        .try_reduce(
            TestFileAnalysis::default,
            |mut left, mut right| -> Result<_> {
                left.edges.append(&mut right.edges);
                left.helper_references.append(&mut right.helper_references);
                Ok(left)
            },
        )
}
