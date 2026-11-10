use super::super::cause::CauseKind;
use super::super::coverage::{assemble, blank, call_site, empty_graph, NoFacts};
use super::super::escapes::Escapes;
use crate::codebase::dependencies::extract::CallableId;
use crate::codebase::dependencies::graph::NodeId;
use crate::codebase::ts_source::module_facts::arguments::{PathAnchor, PathOrigin};
use crate::fx::FxHashMap;
use std::path::PathBuf;

fn parameter_origin(owner: u32) -> PathOrigin {
    PathOrigin {
        anchor: PathAnchor::Unknown,
        segments: Vec::new(),
        dynamic: false,
        env: None,
        param: Some(0),
        param_member: None,
        param_owner: Some(owner),
        owner_line: Some(3),
        reassigned_at: None,
    }
}

#[test]
fn an_escaped_caller_blocks_a_proven_parameter() {
    let enumerator = NodeId::callable("/repo/src/fn.mts", "enumerate", CallableId(11));
    let enum_file = enumerator.as_file().unwrap().to_path_buf();
    let caller_file = NodeId::callable("/repo/tests/t.mts", "testFn", CallableId(2))
        .as_file()
        .unwrap()
        .to_path_buf();
    let caller_site = call_site(&caller_file, "testFn", 2);
    let site = call_site(&enum_file, "enumerate", 11);
    let graph = empty_graph();
    let mut callers = FxHashMap::default();
    callers.insert(enumerator.clone(), vec![&caller_site]);
    let mut nodes: FxHashMap<PathBuf, FxHashMap<u32, NodeId>> = FxHashMap::default();
    nodes.entry(enum_file).or_default().insert(11, enumerator);
    let mut fixture = blank(&graph);
    fixture.escaped = Escapes::with_local(&caller_file, "testFn");
    fixture.callers = callers;
    fixture.nodes = nodes;
    let cause = assemble(fixture)
        .param_cause(0, &parameter_origin(11), &site, &NoFacts, &[])
        .expect_err("caller");
    assert_eq!(cause.kind, CauseKind::NonDirectReference);
    let text = cause.detail(std::path::Path::new("/repo"));
    assert!(text.contains("tests/t.mts:1 references `testFn`"), "{text}");
}
