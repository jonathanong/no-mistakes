use super::*;
#[test]
fn snapshots_share_all_incoming_scope_and_dense_payloads() {
    let (path, _, file) = fixtures::fixture("cyclic");
    let files = support::files(&path, &file.query_annotation[0].1, &file.embedded_sql[0].1);
    let mut evaluator = support::evaluator(&files);
    evaluator.module_environment(&path);
    let fork = evaluator.fork_arm();
    assert_eq!(fork.scopes.len(), evaluator.scopes.len());
    assert!(fork
        .scopes
        .iter()
        .zip(&evaluator.scopes)
        .all(|(a, b)| a.same_bindings(b)));
    assert_eq!(
        fork.argument_objects.len(),
        evaluator.argument_objects.len()
    );
    assert!(!fork.argument_objects.is_empty());
    for slots in fork.argument_objects.values() {
        assert!(slots.shared());
    }
    // The first source-order delta needs no identity translation or payload detachment.
    let translation = super::super::translate::Translation::new(2, 3, 2, 3);
    assert!(translation.identity());
}
