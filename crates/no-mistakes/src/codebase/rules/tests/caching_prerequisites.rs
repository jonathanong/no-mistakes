use super::*;

#[test]
fn aggregate_check_propagates_nextjs_caching_missing_source_error() {
    let root = fixture("codebase-analysis/no-nextjs-caching");
    let shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        vec![root.join("web/app/bad.ts")],
        crate::codebase::check_facts::CheckFactPlan::default(),
    );

    let error = run_check_with_facts(&root, None, None, &shared).unwrap_err();

    assert_eq!(
        error.to_string(),
        format!(
            "nextjs-no-caching requires source facts for {}",
            root.join("web/app/bad.ts").display()
        )
    );
}
