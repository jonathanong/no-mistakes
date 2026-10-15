use super::*;

fn occurrence(offset: u32, target: &str) -> ResolvedCallSite {
    let file = std::path::PathBuf::from("/fixture/edge-multiplicity.mts");
    ResolvedCallSite {
        file: file.clone(),
        caller: None,
        caller_id: None,
        line: 1,
        offset,
        invocation: InvocationKind::Call,
        source_callee: "original".into(),
        target: ResolvedCallTarget::RepositoryFunction {
            file: file.clone(),
            scope: target.into(),
        },
        returned_from: Vec::new(),
        typed_from: Vec::new(),
        returned_member_from: Vec::new(),
        target_node: Some(NodeId::symbol(file, target)),
    }
}
#[test]
fn replacing_one_physical_call_keeps_other_call_and_implicit_edge() {
    let base = [occurrence(10, "original"), occurrence(20, "original")];
    let sites = [occurrence(10, "replacement"), base[1].clone()];
    let source = NodeId::file(&base[0].file);
    let original = base[0].target_node.clone().unwrap();
    let replacement = sites[0].target_node.clone().unwrap();
    // Two source calls plus an implicit constructor Call with equal endpoints.
    let mut edges = vec![(source.clone(), original.clone(), EdgeKind::Call); 3];
    replace_call_edges(&mut edges, &base, &sites, &[0], |_| source.clone());
    assert_eq!(
        edges
            .iter()
            .filter(|(_, target, kind)| target == &original && *kind == EdgeKind::Call)
            .count(),
        2
    );
    assert_eq!(
        edges
            .iter()
            .filter(|(_, target, kind)| target == &replacement && *kind == EdgeKind::Call)
            .count(),
        1
    );
}
#[test]
fn unknown_replacement_removes_only_the_refined_physical_edge() {
    let base = [occurrence(10, "original"), occurrence(20, "original")];
    let mut sites = base.clone();
    sites[0].target = ResolvedCallTarget::Unknown;
    sites[0].target_node = None;
    let source = NodeId::file(&base[0].file);
    let original = base[0].target_node.clone().unwrap();
    let mut edges = vec![(source.clone(), original.clone(), EdgeKind::Call); 2];
    replace_call_edges(&mut edges, &base, &sites, &[0], |_| source.clone());
    assert_eq!(edges, [(source, original, EdgeKind::Call)]);
}
