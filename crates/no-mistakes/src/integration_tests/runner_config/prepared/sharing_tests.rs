use super::*;
use crate::codebase::ts_resolver::VisiblePathLookup;
use std::sync::Arc;

#[test]
fn runner_owner_reuses_visibility_and_resolved_imports_across_config_parses() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config/session-visible-resolver")
        .canonicalize()
        .unwrap();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let visible = session.visible_paths(&root).paths_for(&root);
    let config = session.config(&root, None).unwrap();
    let tsconfig = session.tsconfig(&root, None).unwrap();
    let prepared = prepare(&root, &config, &visible, &tsconfig);
    let first_visible = prepared.visible_files.normalized_visible();
    crate::diagnostics::with_observer(Some(observer.clone()), || {
        let first = prepared.parse_all().unwrap();
        let first_work = observer.snapshot().work;
        assert_eq!(first.files.len(), 2);
        assert!(first_work["resolver.computations"] > 0);
        let repeated = prepared.clone().parse_all().unwrap();
        assert_eq!(repeated.files.len(), 2);
        for facts in repeated.files.values() {
            let projects = facts.vitest_projects().unwrap().unwrap();
            assert_eq!(projects.len(), 1);
            assert!(matches!(
                projects[0].policy_name.as_deref(),
                Some("unit-a" | "unit-b")
            ));
            assert_eq!(projects[0].include, ["tests/**/*.test.mts"]);
        }
        let second_work = observer.snapshot().work;
        assert_eq!(
            second_work["resolver.computations"],
            first_work["resolver.computations"]
        );
        assert!(second_work["resolver.requests"] > first_work["resolver.requests"]);
    });
    assert!(Arc::ptr_eq(
        &first_visible,
        &prepared.clone().visible_files.normalized_visible()
    ));
}
