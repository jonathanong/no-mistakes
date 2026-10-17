use super::*;

fn site(target: ResolvedCallTarget, node: Option<NodeId>) -> ResolvedCallSite {
    ResolvedCallSite {
        file: "entry.mts".into(),
        caller: None,
        caller_id: None,
        line: 1,
        offset: 0,
        invocation: InvocationKind::Call,
        source_callee: "query".into(),
        target,
        returned_from: vec![],
        typed_from: vec![],
        returned_member_from: vec![],
        target_node: node,
        alternatives: vec![],
    }
}
#[test]
fn choices_keep_unknown_and_deduplicate_identical_nodes() {
    let known = site(
        ResolvedCallTarget::Global {
            name: "read".into(),
        },
        None,
    );
    let result = target(&[
        known.clone(),
        known,
        site(ResolvedCallTarget::Unknown, None),
    ]);
    let ResolvedCallTarget::Alternatives(value) = result else {
        panic!("known alternatives")
    };
    assert_eq!(value.known().len(), 1);
    assert!(value.has_unknown);
    assert_eq!(target(&[]), ResolvedCallTarget::Unknown);
    assert_eq!(
        target(&[site(ResolvedCallTarget::Unknown, None)]),
        ResolvedCallTarget::Unknown
    );
}
#[test]
fn choices_preserve_each_module_selector_and_repository_node() {
    let node = NodeId::file("database.mts");
    let imported = site(
        ResolvedCallTarget::ModuleExport {
            specifier: "@vendor/db".into(),
            export_path: "query".into(),
            repository_target: Some(("database.mts".into(), "query".into())),
            callable_id: Some(CallableId(7)),
        },
        Some(node.clone()),
    );
    let local = site(
        ResolvedCallTarget::RepositoryFunction {
            file: "database.mts".into(),
            scope: "query".into(),
        },
        Some(node.clone()),
    );
    let value = target(&[imported, local]);
    let combined = site(value, None);
    assert_eq!(combined.targets().len(), 2);
    assert_eq!(combined.target_nodes().collect::<Vec<_>>(), vec![&node]);
    assert!(!combined.has_unknown_target());
    assert_eq!(target(std::slice::from_ref(&combined)), combined.target);
}
