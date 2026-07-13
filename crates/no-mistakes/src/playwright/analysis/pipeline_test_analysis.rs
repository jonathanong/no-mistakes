use super::context::{DiscoveredTestFile, TestAnalysisContext};
use super::pipeline_occurrences::PreparedTestFile;
use super::test_occurrence_edges::analyze_direct_test_occurrences;
use super::text_edges::{append_locator_text_edges, AppTextIndex, TextEdgeContext};
use super::types::TestFileAnalysis;
use crate::playwright::fsutil::relative_string;
use crate::playwright::test_file_occurrences::TestFileOccurrences;
use rayon::prelude::*;
use std::sync::Arc;

pub(crate) struct PendingTestFileAnalysis {
    test_file: DiscoveredTestFile,
    analysis: TestFileAnalysis,
    occurrences: TestFileOccurrences,
}

pub(crate) fn analyze_direct_test_files(
    prepared: Vec<PreparedTestFile>,
    context: &TestAnalysisContext<'_>,
) -> Vec<PendingTestFileAnalysis> {
    prepared
        .into_par_iter()
        .map(|prepared| {
            let analysis = analyze_direct_test_occurrences(
                &prepared.test_file,
                context,
                &prepared.occurrences,
            );
            PendingTestFileAnalysis {
                test_file: prepared.test_file,
                analysis,
                occurrences: prepared.occurrences,
            }
        })
        .collect()
}

pub(crate) fn has_text_locator_candidate(
    pending: &[PendingTestFileAnalysis],
    app_text_targets: &[super::text_types::AppTextTarget],
    app_text_index: &AppTextIndex,
    test_policy: crate::playwright::playwright_tests::TestPolicy,
) -> bool {
    pending.iter().any(|file| {
        file.occurrences.text_locators().iter().any(|locator| {
            test_policy.allows(locator.status)
                && locator.scope
                    != crate::playwright::playwright_tests::TestOccurrenceScope::TeardownHook
                && super::text_edges::locator_has_app_text_candidate(
                    app_text_targets,
                    app_text_index,
                    &locator.value,
                )
        })
    })
}

pub(crate) fn has_route_reachability_demand(
    root: &std::path::Path,
    pending: &[PendingTestFileAnalysis],
    app_text_targets: &[super::text_types::AppTextTarget],
    app_text_index: &AppTextIndex,
    test_policy: crate::playwright::playwright_tests::TestPolicy,
) -> bool {
    pending.iter().any(|file| {
        let test_file = relative_string(root, &file.test_file.path);
        file.occurrences.text_locators().iter().any(|locator| {
            test_policy.allows(locator.status)
                && locator.scope
                    != crate::playwright::playwright_tests::TestOccurrenceScope::TeardownHook
                && super::text_edges::locator_has_app_text_candidate(
                    app_text_targets,
                    app_text_index,
                    &locator.value,
                )
                && file.analysis.edges.iter().any(|edge| {
                    super::text_edges::route_signal_matches_locator(
                        edge,
                        &test_file,
                        locator.test_name.as_deref(),
                        &locator.describe_path,
                        locator.scope,
                        locator.line,
                    )
                })
        })
    })
}

pub(crate) fn finish_test_file_analysis(
    pending: Vec<PendingTestFileAnalysis>,
    context: &TestAnalysisContext<'_>,
    text_context: Option<&TextEdgeContext<'_>>,
) -> TestFileAnalysis {
    pending
        .into_par_iter()
        .map(|mut pending| {
            if let Some(text_context) = text_context {
                append_locator_text_edges(
                    &mut pending.analysis.edges,
                    &Arc::new(relative_string(context.root, &pending.test_file.path)),
                    &pending.test_file.test_id_attributes(),
                    text_context,
                    pending.occurrences.text_locators(),
                );
            }
            pending.analysis
        })
        .reduce(TestFileAnalysis::default, |mut left, mut right| {
            left.edges.append(&mut right.edges);
            left.helper_references.append(&mut right.helper_references);
            left
        })
}
