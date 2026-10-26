use super::needs_scoped_facts;
use crate::codebase::check_facts::CheckFactPlan;

#[test]
fn low_annotation_heads_keep_remaining_playwright_files_in_scoped_collection() {
    assert!(!needs_scoped_facts(&CheckFactPlan::default()));
    assert!(needs_scoped_facts(&CheckFactPlan {
        query_annotation_heads: true,
        ..Default::default()
    }));
}
