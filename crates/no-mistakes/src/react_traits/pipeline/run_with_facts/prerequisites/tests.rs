use super::*;
use crate::codebase::analysis_session::AnalysisSession;

#[test]
fn prerequisite_failure_preserves_legacy_error_and_partial_consumer() {
    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/partial-failures/fixture/src/broken.ts");
    assert!(file.exists());
    let invoke =
        |session: Option<&AnalysisSession>| accept(session, &[(file.clone(), "invalid source")]);
    let session = AnalysisSession::disabled();
    for error in [
        invoke(None).unwrap_err(),
        invoke(Some(&session)).unwrap_err(),
    ] {
        assert!(error.to_string().contains("failed to parse"));
        assert!(error.to_string().contains("invalid source"));
        assert!(error.to_string().contains(&file.display().to_string()));
    }
    assert!(session.check_failures().is_empty());
    session.enable_partial_checks();
    assert!(!invoke(Some(&session)).unwrap());
    let failures = session.check_failures();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].consumer, "assert-no-fetch");
    assert_eq!(failures[0].file.as_ref(), Some(&file));
    assert!(failures[0].message.contains("invalid source"));
}

#[test]
fn child_prerequisites_tolerate_duplicate_references_and_unprepared_children() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/react-traits-components/partial-prerequisites/fixture");
    let shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        crate::codebase::ts_source::discover_visible_paths(&root),
        crate::codebase::check_facts::CheckFactPlan {
            react: true,
            ..Default::default()
        },
    );
    let mut parent = shared
        .ts
        .iter()
        .find_map(|(path, file)| {
            if !path.ends_with("app/components/Parent.tsx") {
                return None;
            }
            file.react.as_ref()?.components.first().cloned()
        })
        .unwrap();
    assert!(!parent.children.is_empty());
    parent.children.extend(parent.children.clone());
    let sources = source_index(&shared);
    let failures = child_failures(&parent, &root, &sources);
    assert_eq!(failures.len(), 1);
    assert!(failures[0].0.ends_with("Child.tsx"));
    assert!(child_failures(&parent, &root, &HashMap::new()).is_empty());
}
