use super::super::cause::CauseKind;
use super::super::coverage::{assemble, blank, empty_graph, NoFacts};
use super::super::escapes::Escapes;
use crate::codebase::dependencies::extract::CallableId;
use crate::codebase::dependencies::graph::test_support::from_typed_maps;
use crate::codebase::dependencies::graph::{EdgeKind, NodeId};
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

#[test]
fn a_cached_unread_closure_stops_before_the_escape() {
    let source = NodeId::callable("/repo/src/fn.mts", "fn", CallableId(1));
    let module = source.as_file().unwrap().to_path_buf();
    let graph = empty_graph();
    let mut fixture = blank(&graph);
    fixture.escaped = Escapes::with_local(&module, "fn");
    fixture.reached.insert(module, true);
    let cause = assemble(fixture).route_cause(&source, &NoFacts);
    assert_eq!(cause.kind, CauseKind::CallersNotProven);
    let text = cause.detail(Path::new("/repo"));
    assert!(
        text.contains("callers are not proven (src/fn.mts:1 declares it)"),
        "{text}"
    );
}

#[test]
fn a_non_file_dependent_stays_outside_the_closure() {
    let file = PathBuf::from("/repo/src/fn.mts");
    let reverse = FxHashMap::from_iter([(
        NodeId::file(&file),
        vec![(NodeId::module("pkg"), EdgeKind::Import)],
    )]);
    let graph = from_typed_maps(PathBuf::from("/repo"), FxHashMap::default(), reverse);
    let cause = assemble(blank(&graph)).route_cause(&NodeId::file(&file), &NoFacts);
    assert_eq!(cause.kind, CauseKind::NotNamedFunction);
}
