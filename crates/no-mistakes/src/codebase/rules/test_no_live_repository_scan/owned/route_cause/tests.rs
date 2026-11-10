use super::super::cause::CauseKind;
use super::super::consumers::Consumed;
use super::super::coverage::{assemble, blank, empty_graph, NoFacts};
use super::super::escapes::Escapes;
use super::super::setups::ConfiguredSetups;
use crate::codebase::dependencies::extract::CallableId;
use crate::codebase::dependencies::graph::NodeId;
use std::path::Path;

#[test]
fn a_gap_without_a_config_file_is_unreadable() {
    let graph = empty_graph();
    let mut fixture = blank(&graph);
    fixture.setups = ConfiguredSetups::unknown_without_gap(Path::new("/repo"));
    let cause = assemble(fixture).route_cause(&NodeId::module("pkg"), &NoFacts);
    assert_eq!(cause.kind, CauseKind::UnreadableConfig);
    let text = cause.detail(Path::new("/repo"));
    assert!(text.contains("cannot be read or does not match"), "{text}");
}

#[test]
fn a_non_file_source_is_not_a_named_function() {
    let graph = empty_graph();
    let cause = assemble(blank(&graph)).route_cause(&NodeId::module("pkg"), &NoFacts);
    assert_eq!(cause.kind, CauseKind::NotNamedFunction);
    let text = cause.detail(Path::new("/repo"));
    assert!(text.contains("is not a named function"), "{text}");
}

#[test]
fn an_escape_without_a_reference_names_the_function() {
    let source = NodeId::callable("/repo/src/fn.mts", "fn", CallableId(1));
    let file = source.as_file().unwrap().to_path_buf();
    let graph = empty_graph();
    let mut fixture = blank(&graph);
    fixture.escaped = Escapes::with_local(&file, "fn");
    let cause = assemble(fixture).route_cause(&source, &NoFacts);
    assert_eq!(cause.kind, CauseKind::NonDirectReference);
    let text = cause.detail(Path::new("/repo"));
    assert!(text.contains("src/fn.mts:1 references `fn`"), "{text}");
}

#[test]
fn a_consumed_file_without_a_hit_falls_back_to_callers() {
    let source = NodeId::callable("/repo/src/fn.mts", "fn", CallableId(1));
    let file = source.as_file().unwrap().to_path_buf();
    let graph = empty_graph();
    let mut fixture = blank(&graph);
    fixture.consumed = Consumed::containing(&file);
    let cause = assemble(fixture).route_cause(&source, &NoFacts);
    assert_eq!(cause.kind, CauseKind::CallersNotProven);
}
