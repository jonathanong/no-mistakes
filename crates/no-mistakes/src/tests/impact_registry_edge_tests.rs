use super::*;

#[test]
fn registry_hints_ignore_non_runtime_dependency_edges() {
    for kind in [
        EdgeKind::TypeImport,
        EdgeKind::WorkspaceTypeImport,
        EdgeKind::RequireResolve,
    ] {
        assert!(!is_runtime_registration_edge(&kind));
    }
    assert!(is_runtime_registration_edge(&EdgeKind::Import));
    assert!(is_runtime_registration_edge(&EdgeKind::WorkspaceImport));
    assert!(is_runtime_registration_edge(&EdgeKind::Require));
}
