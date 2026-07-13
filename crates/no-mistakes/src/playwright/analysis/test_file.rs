use super::context::{DiscoveredTestFile, TestAnalysisContext};
use super::pipeline_occurrences::extract_test_file_occurrences;
use super::test_occurrence_edges::analyze_prepared_test_occurrences;
use super::types::TestFileAnalysis;
use crate::playwright::test_file_occurrences::TestFileOccurrences;
use anyhow::Result;

pub(crate) fn analyze_test_file(
    test_file: &DiscoveredTestFile,
    context: &TestAnalysisContext<'_>,
) -> Result<TestFileAnalysis> {
    let occurrences = extract_test_file_occurrences(
        test_file,
        context.navigation_helpers,
        context.selector_regexes,
    )?;
    Ok(analyze_prepared_test_occurrences(
        test_file,
        context,
        &occurrences,
    ))
}

pub(crate) fn analyze_test_occurrences(
    test_file: &DiscoveredTestFile,
    context: &TestAnalysisContext<'_>,
    occurrences: &TestFileOccurrences,
) -> TestFileAnalysis {
    analyze_prepared_test_occurrences(test_file, context, occurrences)
}
